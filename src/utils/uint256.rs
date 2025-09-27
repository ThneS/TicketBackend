use alloy::primitives::Uint;
use bigdecimal::BigDecimal;
use hex;
use num_bigint::{BigInt, BigUint, Sign};
use sqlx::{
    Postgres, Type,
    decode::Decode,
    encode::Encode,
    postgres::{PgHasArrayType, PgTypeInfo, PgValueRef},
    types::BigDecimal as SqlxBigDecimal,
};
use std::{fmt, str::FromStr};

/// 数据库存储用的本地包装类型，解决 orphan rule：为本地类型实现外部 trait
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DbU256(pub U256);

impl From<u64> for DbU256 {
    fn from(v: u64) -> Self {
        DbU256(U256::from(v))
    }
}

impl serde::Serialize for DbU256 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // 以十进制字符串序列化，便于调试与前端消费
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for DbU256 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        DbU256::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl fmt::Display for DbU256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for DbU256 {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err("empty string".into());
        }
        if let Some(rest) =
            s.strip_prefix("0x").or_else(|| s.strip_prefix("0X"))
        {
            // hex
            DbU256::from_hex_str(rest)
        } else {
            // decimal
            let big = BigUint::from_str(s).map_err(|e| e.to_string())?;
            Ok(DbU256(biguint_to_u256(&big)))
        }
    }
}

impl DbU256 {
    /// 以 0x 前缀的紧凑 hex 字符串（去除前导 0），零值为 0x0
    pub fn to_hex0x(&self) -> String {
        let bytes = self.0.to_be_bytes::<32>();
        let mut first = 0usize;
        while first < bytes.len() && bytes[first] == 0 {
            first += 1;
        }
        if first == bytes.len() {
            return "0x0".to_string();
        }
        format!("0x{}", hex::encode(&bytes[first..]))
    }

    /// 从 hex 字符串构建（可带 0x/0X，或纯十六进制，不区分大小写）
    pub fn from_hex_str(s: &str) -> Result<Self, String> {
        let h = s
            .strip_prefix("0x")
            .or_else(|| s.strip_prefix("0X"))
            .unwrap_or(s);
        if h.is_empty() {
            return Err("empty hex".into());
        }
        // 处理奇数位 hex：前置补 0
        let h_norm: std::borrow::Cow<str> = if h.len() % 2 == 1 {
            let mut s2 = String::with_capacity(h.len() + 1);
            s2.push('0');
            s2.push_str(h);
            std::borrow::Cow::Owned(s2)
        } else {
            std::borrow::Cow::Borrowed(h)
        };
        // decode 为 bytes；再左侧填充到 32 字节
        let bytes = match hex::decode(h_norm.as_ref()) {
            Ok(b) => b,
            Err(e) => return Err(e.to_string()),
        };
        Ok(DbU256(U256::from_be_bytes::<32>({
            let mut padded = [0u8; 32];
            if bytes.len() > 32 {
                return Err("hex too long for U256".into());
            }
            let offset = 32 - bytes.len();
            padded[offset..].copy_from_slice(&bytes);
            padded
        })))
    }
}

/// 可选的 serde 模块：将 DbU256 序列化为十六进制字符串（0x 开头），并从中反序列化
pub mod serde_hex {
    use super::*;
    use serde::{Deserialize, Deserializer, Serializer, de::Error as DeError};

    pub fn serialize<S>(
        value: &DbU256,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&value.to_hex0x())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DbU256, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        DbU256::from_hex_str(&s).map_err(D::Error::custom)
    }
}

/// EVM 的 uint256 类型别名
pub type U256 = Uint<256, 4>;

/// Uint256 -> BigUint
pub fn u256_to_biguint(value: U256) -> BigUint {
    BigUint::from_bytes_be(&value.to_be_bytes::<32>())
}

/// BigUint -> Uint256
pub fn biguint_to_u256(value: &BigUint) -> U256 {
    let bytes = value.to_bytes_be();
    let mut padded = [0u8; 32];
    let offset = 32 - bytes.len();
    padded[offset..].copy_from_slice(&bytes);
    U256::from_be_bytes::<32>(padded)
}

/// Uint256 -> BigDecimal
pub fn u256_to_bigdecimal(value: U256) -> BigDecimal {
    // 直接通过 BigInt 构造，避免字符串中转
    let bu = u256_to_biguint(value);
    let bi = BigInt::from(bu);
    BigDecimal::from(bi)
}

/// BigDecimal -> Uint256
pub fn bigdecimal_to_u256(value: &BigDecimal) -> U256 {
    // 使用严格路径，避免字符串中转
    DbU256::try_from(value).unwrap().0
}

/// BigUint -> BigDecimal
pub fn biguint_to_bigdecimal(value: &BigUint) -> BigDecimal {
    BigDecimal::from_str(&value.to_str_radix(10)).unwrap()
}

/// BigDecimal -> BigUint
pub fn bigdecimal_to_biguint(value: &BigDecimal) -> BigUint {
    BigUint::from_str(&value.to_string()).unwrap()
}

/// 实现 sqlx::Type (告诉 sqlx 在 PG 里对应 NUMERIC) —— 针对本地包装类型 DbU256
impl Type<Postgres> for DbU256 {
    fn type_info() -> PgTypeInfo {
        <SqlxBigDecimal as Type<Postgres>>::type_info()
    }
}

impl PgHasArrayType for DbU256 {
    fn array_type_info() -> PgTypeInfo {
        <SqlxBigDecimal as PgHasArrayType>::array_type_info()
    }
}

/// Encode (写数据库) —— 将 DbU256 以 BigDecimal 编码到 NUMERIC
impl<'q> Encode<'q, Postgres> for DbU256 {
    fn encode_by_ref(
        &self,
        buf: &mut <Postgres as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>>
    {
        // 通过 BigInt -> BigDecimal 构造，确保 scale=0，避免字符串中转
        let bu = u256_to_biguint(self.0);
        let bi = BigInt::from(bu);
        let bd = BigDecimal::from(bi);
        bd.encode_by_ref(buf)
    }
}

/// 严格的 BigDecimal -> DbU256 转换（尽量避免字符串中转）
impl TryFrom<&BigDecimal> for DbU256 {
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn try_from(bd: &BigDecimal) -> Result<Self, Self::Error> {
        // 判定是否为整数：若与向零取整后的值相等（按数值相等），则认为是整数（包括 123.000）
        let scaled_down = bd.with_scale(0); // 向零取整
        if &scaled_down != bd {
            return Err("fractional numeric not supported for U256".into());
        }

        // 现在是整数，取其非负校验并转为 BigInt/bytes
        // BigDecimal -> BigInt 最稳妥的方法：字符串中转或通过 to_bytes_be (不可用)。
        // 为保证兼容 bigdecimal 0.4，我们使用最小字符串中转但经过上面的整数性校验。
        let s = bd.with_scale(0).to_string();
        let bi = BigInt::from_str(&s)?;
        big_int_to_db_u256(bi)
    }
}

impl TryFrom<BigDecimal> for DbU256 {
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn try_from(bd: BigDecimal) -> Result<Self, Self::Error> {
        DbU256::try_from(&bd)
    }
}

fn big_int_to_db_u256(
    bi: BigInt,
) -> Result<DbU256, Box<dyn std::error::Error + Send + Sync>> {
    match bi.to_bytes_be() {
        (Sign::Minus, _) => {
            Err("negative numeric cannot be represented as U256".into())
        }
        (_, bytes) => {
            if bytes.len() > 32 {
                return Err("numeric too large for U256".into());
            }
            let mut padded = [0u8; 32];
            let offset = 32 - bytes.len();
            padded[offset..].copy_from_slice(&bytes);
            Ok(DbU256(U256::from_be_bytes::<32>(padded)))
        }
    }
}

/// Decode (读数据库) —— 从 NUMERIC 解码为 DbU256
impl<'r> Decode<'r, Postgres> for DbU256 {
    fn decode(
        value: PgValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // 从 NUMERIC 取出 BigDecimal
        let big_decimal = <SqlxBigDecimal as Decode<Postgres>>::decode(value)?;
        DbU256::try_from(&big_decimal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fromstr_decimal_and_hex() {
        let d = DbU256::from_str("123456").unwrap();
        let h = DbU256::from_str("0x1e240").unwrap();
        assert_eq!(d.to_string(), h.to_string());
    }

    #[test]
    fn test_hex_render_and_parse() {
        let v = DbU256::from_str("0").unwrap();
        assert_eq!(v.to_hex0x(), "0x0");
        let v2 = DbU256::from_str("0x0").unwrap();
        assert_eq!(v.to_string(), v2.to_string());

        let x = DbU256::from_str("65535").unwrap();
        assert_eq!(x.to_hex0x(), "0xffff");
        let back = DbU256::from_hex_str("0xffff").unwrap();
        assert_eq!(x.to_string(), back.to_string());
    }

    #[test]
    fn test_serde_decimal_default() {
        let v = DbU256::from_str("1000000000000000000").unwrap();
        let s = serde_json::to_string(&v).unwrap();
        // 默认十进制字符串
        assert_eq!(s, "\"1000000000000000000\"");
        let de: DbU256 = serde_json::from_str(&s).unwrap();
        assert_eq!(v.to_string(), de.to_string());
    }

    #[test]
    fn test_serde_hex_module() {
        #[derive(serde::Serialize, serde::Deserialize)]
        struct HexWrap(
            #[serde(with = "crate::utils::uint256::serde_hex")] DbU256,
        );

        let v = DbU256::from_str("255").unwrap();
        let s = serde_json::to_string(&HexWrap(v.clone())).unwrap();
        assert_eq!(s, "\"0xff\"");
        let HexWrap(back) = serde_json::from_str(&s).unwrap();
        assert_eq!(v.to_string(), back.to_string());
    }

    #[test]
    fn test_bigdecimal_to_db_u256_strict() {
        // valid integer-like inputs
        let cases = vec![
            ("0", "0"),
            ("+0", "0"),
            ("0000", "0"),
            ("123", "123"),
            ("+123", "123"),
            ("000123", "123"),
            ("123.0", "123"),
            ("123.000000", "123"),
        ];
        for (s, expect) in cases {
            let bd = BigDecimal::from_str(s).unwrap();
            let v = DbU256::try_from(&bd).unwrap();
            assert_eq!(v.to_string(), expect);
        }

        // negative
        let neg = BigDecimal::from_str("-1").unwrap();
        assert!(DbU256::try_from(&neg).is_err());

        // fractional non-zero
        let frac = BigDecimal::from_str("1.5").unwrap();
        assert!(DbU256::try_from(&frac).is_err());
    }
}
