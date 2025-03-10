use secrecy::zeroize::Zeroizing;

// Constants for markers
const BEGIN_MARKER: &[u8] = b"---   BEGIN ENCRYPTED DATA    ---";
const END_MARKER: &[u8] = b"---   END ENCRYPTED DATA    ---";

// Function to wrap data with markers before encryption
fn wrap_with_markers(data: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();
    
    // Add beginning marker
    result.extend_from_slice(BEGIN_MARKER);
    result.extend_from_slice(b"\n");
    
    // Add the actual data
    result.extend_from_slice(data);
    result.extend_from_slice(b"\n");
    
    // Add ending marker
    result.extend_from_slice(END_MARKER);
    
    result
}

// Function to extract data between markers after decryption
fn extract_from_markers(data: &mut Zeroizing<Vec<u8>>) -> Zeroizing<Vec<u8>> {
    // Convert to string for easier manipulation
    // (assuming UTF-8 encoding works for your data)
    let data_str = String::from_utf8_lossy(data.as_slice());
    
    // Find the positions of the markers
    if let (Some(begin_pos), Some(end_pos)) = (
        data_str.find(std::str::from_utf8(BEGIN_MARKER).unwrap_or("")),
        data_str.find(std::str::from_utf8(END_MARKER).unwrap_or(""))
    ) {
        // Calculate the start and end of the actual data
        let data_start = begin_pos + BEGIN_MARKER.len() + 1; // +1 for newline
        let data_end = end_pos.saturating_sub(1); // -1 for newline
        
        // Extract the data if the markers are in the correct order
        if data_start < data_end {
            return Zeroizing::new(data_str[data_start..data_end].as_bytes().to_vec());
        }
    }
    
    // Fallback: binary search for markers if UTF-8 conversion fails
    if let Some(begin_idx) = find_subsequence(data, BEGIN_MARKER) {
        if let Some(end_idx) = find_subsequence(data, END_MARKER) {
            let start = begin_idx + BEGIN_MARKER.len() + 1; // +1 for newline
            let end = end_idx.saturating_sub(1); // -1 for newline
            
            if start < end {
                return Zeroizing::new(data[start..end].to_vec());
            }
        }
    }
    
    // If markers weren't found or in wrong order, return empty
    Zeroizing::new(Vec::new())
}

// Helper function to find a subsequence in a byte slice
fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len())
        .position(|window| window == needle)
}

// Combined function for the encryption flow
fn prepare_for_encryption(plaintext: &[u8]) -> Vec<u8> {
    wrap_with_markers(plaintext)
}

// Combined function for the decryption flow
fn process_after_decryption(data: &mut Zeroizing<Vec<u8>>) -> Zeroizing<Vec<u8>> {
    extract_from_markers(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_wrap_with_markers() {
        // Test with simple data
        let input = b"test data";
        let wrapped = wrap_with_markers(input);
        
        // Check that the wrapped data has the correct format
        assert!(wrapped.starts_with(BEGIN_MARKER));
        assert!(wrapped.ends_with(END_MARKER));
        
        // Check that our data is in between
        let wrapped_str = String::from_utf8_lossy(&wrapped);
        assert!(wrapped_str.contains("test data"));
    }
    
    #[test]
    fn test_wrap_with_markers_empty() {
        // Test with empty data
        let input = b"";
        let wrapped = wrap_with_markers(input);
        
        // Check that the markers are still present
        assert!(wrapped.starts_with(BEGIN_MARKER));
        assert!(wrapped.ends_with(END_MARKER));
        
        // Expected format: BEGIN_MARKER + \n + \n + END_MARKER
        let expected_len = BEGIN_MARKER.len() + 1 + 1 + END_MARKER.len();
        assert_eq!(wrapped.len(), expected_len);
    }
    
    #[test]
    fn test_extract_from_markers() {
        // Create properly wrapped data
        let original = b"secret message";
        let wrapped = wrap_with_markers(original);
        
        // Extract the data
        let mut wrapped_copy = Zeroizing::new(wrapped.clone());
        let extracted = extract_from_markers(&mut wrapped_copy);
        
        // Check that the extracted data matches the original
        assert_eq!(extracted.as_slice(), original);
    }
    
    #[test]
    fn test_extract_from_markers_empty() {
        // Test with empty payload
        let wrapped = wrap_with_markers(b"");
        let mut wrapped_copy = Zeroizing::new(wrapped.clone());
        let extracted = extract_from_markers(&mut wrapped_copy);
        
        // Should extract empty data
        assert_eq!(extracted.as_slice(), b"");
    }
    
    #[test]
    fn test_extract_from_markers_no_markers() {
        // Test with data without markers
        let mut data = Zeroizing::new(b"just some text without markers".to_vec());
        let extracted = extract_from_markers(&mut data);
        
        // Should return empty when no markers present
        assert!(extracted.is_empty());
    }
    
    #[test]
    fn test_extract_from_markers_incomplete() {
        // Test with only beginning marker
        let mut data_begin_only = Zeroizing::new([BEGIN_MARKER, b"data"].concat());
        let extracted1 = extract_from_markers(&mut data_begin_only);
        assert!(extracted1.is_empty());
        
        // Test with only ending marker
        let mut data_end_only = Zeroizing::new([b"data", END_MARKER].concat());
        let extracted2 = extract_from_markers(&mut data_end_only);
        assert!(extracted2.is_empty());
    }
    
    #[test]
    fn test_extract_from_markers_reversed() {
        // Test with markers in wrong order
        let mut reversed = Zeroizing::new([END_MARKER, b"\ndata\n", BEGIN_MARKER].concat());
        let extracted = extract_from_markers(&mut reversed);
        
        // Should return empty when markers are in wrong order
        assert!(extracted.is_empty());
    }
    
    #[test]
    fn test_find_subsequence() {
        // Basic case
        assert_eq!(find_subsequence(b"hello world", b"world"), Some(6));
        
        // At the beginning
        assert_eq!(find_subsequence(b"hello world", b"hello"), Some(0));
        
        // Not found
        assert_eq!(find_subsequence(b"hello world", b"universe"), None);
        
        // Empty needle always matches at position 0
        assert_eq!(find_subsequence(b"hello world", b""), Some(0));
        
        // Empty haystack only matches empty needle
        assert_eq!(find_subsequence(b"", b""), Some(0));
        assert_eq!(find_subsequence(b"", b"hello"), None);
    }
    
    #[test]
    fn test_prepare_for_encryption() {
        // Since prepare_for_encryption just calls wrap_with_markers,
        // we can test that they return the same result
        let input = b"encrypt me";
        assert_eq!(prepare_for_encryption(input), wrap_with_markers(input));
    }
    
    #[test]
    fn test_process_after_decryption() {
        // Since process_after_decryption just calls extract_from_markers,
        // we can test that they return the same result
        let wrapped = wrap_with_markers(b"decrypted data");
        let mut data1 = Zeroizing::new(wrapped.clone());
        let mut data2 = Zeroizing::new(wrapped);
        
        assert_eq!(
            process_after_decryption(&mut data1).as_slice(),
            extract_from_markers(&mut data2).as_slice()
        );
    }
    
    #[test]
    fn test_full_roundtrip() {
        // Test the complete encrypt/decrypt flow
        let original = b"this is a secret message";
        
        // Encrypt flow
        let prepared = prepare_for_encryption(original);
        
        // Decrypt flow (simulated)
        let mut decrypted_data = Zeroizing::new(prepared);
        let final_data = process_after_decryption(&mut decrypted_data);
        
        // Check roundtrip success
        assert_eq!(final_data.as_slice(), original);
    }
}