# check_nif
Checks if a NIF (Número de Identificação Fiscal, Portuguese Tax Number) is valid

## Validation Methods

- **Validation via nif.pt:** Online lookup to check if the NIF is active and get entity information.
- **Local validation (algorithm):** Checks if the NIF is valid only by calculating the check digit and public rules, without external lookup.

### Local validation

A NIF is considered valid if:
- It has 9 digits.
- The first digit (or first two digits, in the case of "45") is within the allowed values.
- The check digit (9th digit) matches the one calculated by the module 11 algorithm.

Example usage in Rust:
```rust
fn main() {
    let nif = "123456789";
    if is_nif_valid_local(nif) {
        println!("NIF {} is valid!", nif);
    } else {
        println!("NIF {} is invalid!", nif);
    }
}
```
