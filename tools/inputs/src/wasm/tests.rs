use super::*;

fn module(sections: &[(u8, &[u8])]) -> Vec<u8> {
    let mut bytes = b"\0asm\x01\0\0\0".to_vec();
    for &(id, content) in sections {
        assert!(content.len() < 128);
        bytes.extend([id, u8::try_from(content.len()).unwrap()]);
        bytes.extend(content);
    }
    bytes
}

fn import(namespace: &str, kind: u8, descriptor: &[u8]) -> Vec<u8> {
    let mut bytes = vec![1, u8::try_from(namespace.len()).unwrap()];
    bytes.extend(namespace.as_bytes());
    bytes.push(10);
    bytes.extend(b"random_get");
    bytes.push(kind);
    bytes.extend(descriptor);
    bytes
}

#[test]
fn function_imports_memory_tables_and_actual_body_declarations_are_measured() -> Result<()> {
    let imports = import("wasi_snapshot_preview1", 0, &[0]);
    let bytes = module(&[
        (1, &[2, 0x60, 2, 0x7f, 0x7f, 1, 0x7f, 0x60, 0, 0]),
        (2, &imports),
        (3, &[1, 1]),
        (4, &[1, 0x70, 1, 2, 5]),
        (5, &[1, 0, 1]),
        (6, &[1, 0x7f, 0, 0x41, 0, 0x0b]),
        (9, &[0]),
        (10, &[1, 4, 1, 3, 0x7f, 0x0b]),
        (11, &[0]),
    ]);
    let report = declarations(&bytes)?;
    check_wash(&report)?;
    assert_eq!(report["types"], 2);
    assert_eq!(report["max_function_parameters"], 2);
    assert_eq!(report["max_function_results"], 1);
    assert_eq!(
        report["imports"],
        json!([{"module":"wasi_snapshot_preview1","name":"random_get","kind":0,"type":0}])
    );
    assert_eq!(report["defined_functions"], 1);
    assert_eq!(
        report["tables"],
        json!([{"element_type":0x70,"flags":1,"minimum":2,"maximum":5}])
    );
    assert_eq!(report["memories"], json!([{"flags":0,"minimum":1}]));
    assert_eq!(report["globals"], 1);
    assert_eq!(report["element_segments"], 0);
    assert_eq!(report["data_segments"], 0);
    assert_eq!(report["largest_function_body_bytes"], 4);
    assert_eq!(report["max_declared_locals"], 3);
    Ok(())
}

#[test]
fn wash_rejects_foreign_imports_and_nonfunction_wasi_imports() -> Result<()> {
    for (namespace, kind, descriptor) in [
        ("native_host", 0, vec![0]),
        ("wasi_snapshot_preview1", 1, vec![0x70, 0, 1]),
        ("wasi_snapshot_preview1", 2, vec![0, 1]),
        ("wasi_snapshot_preview1", 3, vec![0x7f, 0]),
        ("wasi_snapshot_preview1", 4, vec![0, 0]),
    ] {
        let imports = import(namespace, kind, &descriptor);
        let report = declarations(&module(&[(2, &imports), (5, &[1, 0, 1])]))?;
        assert!(check_wash(&report).is_err());
    }
    Ok(())
}

#[test]
fn wash_rejects_missing_extra_larger_and_bounded_memories() -> Result<()> {
    for bytes in [
        module(&[]),
        module(&[(5, &[0])]),
        module(&[(5, &[2, 0, 1, 0, 1])]),
        module(&[(5, &[1, 0, 2])]),
        module(&[(5, &[1, 1, 1, 16])]),
    ] {
        assert!(check_wash(&declarations(&bytes)?).is_err());
    }
    Ok(())
}

#[test]
fn integer_widths_overflow_and_truncation_are_checked() -> Result<()> {
    assert_eq!(Reader(&[255, 255, 255, 255, 15]).u32()?, u32::MAX);
    assert_eq!(Reader(&[128, 0]).u32()?, 0); // Wasm permits padded encodings.
    assert_eq!(
        Reader(&[255, 255, 255, 255, 255, 255, 255, 255, 255, 1]).unsigned(64)?,
        u64::MAX
    );
    for bytes in [
        &[255, 255, 255, 255, 16][..],
        &[128, 128, 128, 128, 128],
        &[128],
    ] {
        assert!(Reader(bytes).u32().is_err());
    }
    assert!(Reader(&[255; 10]).unsigned(64).is_err());
    Ok(())
}

#[test]
fn memory64_limits_are_not_truncated_and_invalid_limits_fail() -> Result<()> {
    assert_eq!(
        Reader(&[4, 128, 128, 128, 128, 16]).limits()?,
        json!({"flags":4,"minimum":1u64<<32})
    );
    for bytes in [&[1, 2, 1][..], &[2, 1], &[8, 1]] {
        assert!(Reader(bytes).limits().is_err());
    }
    Ok(())
}

#[test]
fn malformed_declarations_duplicates_and_count_mismatches_fail() {
    let mut wrong_version = module(&[]);
    wrong_version[4] = 2;
    let mut overflowing_length = module(&[]);
    overflowing_length.extend([5, 255, 255, 255, 255, 16]);
    for bytes in [
        b"short".to_vec(),
        wrong_version,
        overflowing_length,
        module(&[(1, &[1, 0x5f])]),
        module(&[(2, &[1, 1, 255, 0, 0, 0])]),
        module(&[(2, &import("wasi_snapshot_preview1", 255, &[]))]),
        module(&[(5, &[0, 0])]),
        module(&[(5, &[0]), (5, &[0])]),
        module(&[(5, &[1])]),
        module(&[(14, &[])]),
        module(&[(3, &[1, 0])]),
        module(&[(10, &[1, 1, 0])]),
        module(&[(3, &[1, 0]), (10, &[1, 2, 1, 1])]),
    ] {
        assert!(declarations(&bytes).is_err(), "accepted {bytes:?}");
    }
}

#[test]
fn instruction_validity_is_outside_the_declaration_inspection() -> Result<()> {
    let bytes = module(&[
        (1, &[1, 0x60, 0, 0]),
        (3, &[1, 0]),
        (5, &[1, 0, 1]),
        (10, &[1, 2, 0, 255]), // Invalid instruction; declarations are intact.
    ]);
    let report = declarations(&bytes)?;
    check_wash(&report)?;
    assert_eq!(report["largest_function_body_bytes"], 2);
    assert_eq!(
        report["inspection_scope"],
        "declarations only; not module validation or execution"
    );
    Ok(())
}
