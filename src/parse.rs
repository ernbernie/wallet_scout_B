use crate::errors::{Result, ScoutError};

/// Zero-copy view of SPL Token account data
///
/// Layout (SPL Token Account v2):
/// [0..32]   = mint (32 bytes)
/// [32..64]  = owner (32 bytes)
/// [64..72]  = amount (8 bytes, little-endian)
/// [72..73]  = delegate_option (1 byte)
/// [73..105] = delegate (32 bytes, if delegate_option == 1)
/// [105..106] = state (1 byte)
/// [106..107] = is_native_option (1 byte)
/// [107..115] = is_native (8 bytes, if is_native_option == 1)
/// [115..123] = delegated_amount (8 bytes, little-endian)
/// [123..124] = close_authority_option (1 byte)
/// [124..156] = close_authority (32 bytes, if close_authority_option == 1)
#[derive(Debug, Clone, Copy)]
pub struct TokenAccountView<'a> {
    pub mint: &'a [u8; 32],
    pub owner: &'a [u8; 32],
    pub amount: u64,
    pub delegate_option: u8,
    pub delegate: Option<&'a [u8; 32]>,
    pub state: u8,
    pub is_native_option: u8,
    pub is_native: Option<u64>,
    pub delegated_amount: u64,
    pub close_authority_option: u8,
    pub close_authority: Option<&'a [u8; 32]>,
    pub raw: &'a [u8],
}

/// Parse SPL Token account with strict bounds checking
///
/// Returns a zero-copy view that borrows from the input bytes.
/// No allocations during parsing - only at display time.
pub fn parse_spl_token_account(bytes: &[u8]) -> Result<TokenAccountView<'_>> {
    const MIN_LENGTH: usize = 72; // Minimum for basic fields

    if bytes.len() < MIN_LENGTH {
        return Err(ScoutError::DataTooShort {
            expected: MIN_LENGTH,
            got: bytes.len(),
        });
    }

    // Extract basic fields with bounds checks
    let mint = bytes[0..32].try_into().map_err(|_| ScoutError::Decode {
        field: "mint".to_string(),
        expected: "32 bytes".to_string(),
        got: format!("{} bytes", bytes.len().min(32)),
    })?;

    let owner = bytes[32..64].try_into().map_err(|_| ScoutError::Decode {
        field: "owner".to_string(),
        expected: "32 bytes".to_string(),
        got: format!("{} bytes", (bytes.len() - 32).min(32)),
    })?;

    let amount = u64::from_le_bytes(bytes[64..72].try_into().map_err(|_| ScoutError::Decode {
        field: "amount".to_string(),
        expected: "8 bytes".to_string(),
        got: format!("{} bytes", (bytes.len() - 64).min(8)),
    })?);

    // Check if we have enough data for optional fields
    let delegate_option = if bytes.len() > 72 { bytes[72] } else { 0 };
    let delegate = if delegate_option == 1 && bytes.len() >= 105 {
        Some(bytes[73..105].try_into().map_err(|_| ScoutError::Decode {
            field: "delegate".to_string(),
            expected: "32 bytes".to_string(),
            got: format!("{} bytes", (bytes.len() - 73).min(32)),
        })?)
    } else {
        None
    };

    let state = if bytes.len() > 105 { bytes[105] } else { 0 };
    let is_native_option = if bytes.len() > 106 { bytes[106] } else { 0 };

    let is_native = if is_native_option == 1 && bytes.len() >= 115 {
        Some(u64::from_le_bytes(bytes[107..115].try_into().map_err(
            |_| ScoutError::Decode {
                field: "is_native".to_string(),
                expected: "8 bytes".to_string(),
                got: format!("{} bytes", (bytes.len() - 107).min(8)),
            },
        )?))
    } else {
        None
    };

    let delegated_amount = if bytes.len() >= 123 {
        u64::from_le_bytes(bytes[115..123].try_into().map_err(|_| ScoutError::Decode {
            field: "delegated_amount".to_string(),
            expected: "8 bytes".to_string(),
            got: format!("{} bytes", (bytes.len() - 115).min(8)),
        })?)
    } else {
        0
    };

    let close_authority_option = if bytes.len() > 123 { bytes[123] } else { 0 };
    let close_authority = if close_authority_option == 1 && bytes.len() >= 156 {
        Some(bytes[124..156].try_into().map_err(|_| ScoutError::Decode {
            field: "close_authority".to_string(),
            expected: "32 bytes".to_string(),
            got: format!("{} bytes", (bytes.len() - 124).min(32)),
        })?)
    } else {
        None
    };

    Ok(TokenAccountView {
        mint,
        owner,
        amount,
        delegate_option,
        delegate,
        state,
        is_native_option,
        is_native,
        delegated_amount,
        close_authority_option,
        close_authority,
        raw: bytes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_token_account() {
        // Minimal 72-byte account data
        let mut data = vec![0u8; 72];
        data[64..72].copy_from_slice(&12345u64.to_le_bytes());

        let view = parse_spl_token_account(&data).unwrap();
        assert_eq!(view.amount, 12345);
        assert_eq!(view.delegate_option, 0);
        assert_eq!(view.state, 0);
    }

    #[test]
    fn test_parse_too_short() {
        let data = vec![0u8; 50];
        let result = parse_spl_token_account(&data);
        assert!(matches!(
            result,
            Err(ScoutError::DataTooShort {
                expected: 72,
                got: 50
            })
        ));
    }
}
