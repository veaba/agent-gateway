//! EncryptionService integration tests

use std::path::PathBuf;
use agw_core::security::EncryptionService;

fn temp_key_path() -> PathBuf {
    tempfile::tempdir().unwrap().into_path().join("key.bin")
}

#[test]
fn test_from_key_file_generates_new_key() {
    let path = temp_key_path();
    assert!(!path.exists());

    let _service = EncryptionService::from_key_file(path.clone()).unwrap();
    assert!(path.exists());

    // Key file should be 32 bytes
    let key_data = std::fs::read(&path).unwrap();
    assert_eq!(key_data.len(), 32);
}

#[test]
fn test_from_key_file_loads_existing_key() {
    let path = temp_key_path();

    // First call generates key
    let service1 = EncryptionService::from_key_file(path.clone()).unwrap();
    let encrypted = service1.encrypt("hello world").unwrap();

    // Second call loads same key
    let service2 = EncryptionService::from_key_file(path.clone()).unwrap();
    let decrypted = service2.decrypt(&encrypted).unwrap();

    assert_eq!(decrypted, "hello world");
}

#[test]
fn test_encrypt_decrypt_round_trip() {
    let key = agw_core::security::generate_key();
    let service = EncryptionService::new(&key);

    let plaintext = "secret message";
    let encrypted = service.encrypt(plaintext).unwrap();
    let decrypted = service.decrypt(&encrypted).unwrap();

    assert_eq!(decrypted, plaintext);
    // Encrypted should be base64 and different from plaintext
    assert_ne!(encrypted, plaintext);
}

#[test]
fn test_encrypt_empty_string() {
    let key = agw_core::security::generate_key();
    let service = EncryptionService::new(&key);

    let encrypted = service.encrypt("").unwrap();
    let decrypted = service.decrypt(&encrypted).unwrap();

    assert_eq!(decrypted, "");
}

#[test]
fn test_encrypt_large_plaintext() {
    let key = agw_core::security::generate_key();
    let service = EncryptionService::new(&key);

    let plaintext = "a".repeat(10000);
    let encrypted = service.encrypt(&plaintext).unwrap();
    let decrypted = service.decrypt(&encrypted).unwrap();

    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_encrypt_unicode() {
    let key = agw_core::security::generate_key();
    let service = EncryptionService::new(&key);

    let plaintext = "Hello, 世界! Emojis: 🎉🚀🔒";
    let encrypted = service.encrypt(plaintext).unwrap();
    let decrypted = service.decrypt(&encrypted).unwrap();

    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_decrypt_invalid_data() {
    let key = agw_core::security::generate_key();
    let service = EncryptionService::new(&key);

    // Too short
    let result = service.decrypt("abcd");
    assert!(result.is_err());

    // Invalid base64
    let result = service.decrypt("!!!");
    assert!(result.is_err());
}

#[test]
fn test_key_persistence() {
    let dir = tempfile::tempdir().unwrap();
    let key_path = dir.path().join("subdir").join("key.bin");

    let service = EncryptionService::from_key_file(key_path.clone()).unwrap();
    let encrypted = service.encrypt("persistent secret").unwrap();

    // Verify file exists in subdirectory
    assert!(key_path.exists());

    // Load again and decrypt
    let service2 = EncryptionService::from_key_file(key_path).unwrap();
    let decrypted = service2.decrypt(&encrypted).unwrap();
    assert_eq!(decrypted, "persistent secret");
}
