// main.rs

use reqwest::blocking::Client; // For making synchronous HTTP requests
use scraper::{Html, Selector}; // For parsing HTML

/// Represents the possible outcomes of a NIF query.
#[derive(Debug)]
pub enum NifStatus {
    ValidKnown,      // Valid NIF and known entity
    ValidUnknown,    // Valid NIF but unknown entity
    Error,           // Error message found (invalid NIF)
    MultipleResults, // Multiple companies, NIF not available [Only seen with "000000000"]
    Unknown,         // Could not determine status
}

/// Queries nif.pt with a given NIF number and checks for success, error, or multiple results.
///
/// Returns:
/// - `NifStatus::Success` if a valid company is found.
/// - `NifStatus::Error` if an error message is found.
/// - `NifStatus::MultipleResults` if multiple companies are listed, NIF unavailable.
/// - `NifStatus::Unknown` for request/parse errors or unhandled cases.
pub fn check_nif_status(nif_number: &str) -> NifStatus {
    // Construct the URL for the NIF query
    let url = format!("https://www.nif.pt/?q={}", nif_number);
    println!("Querying URL: {}", url);

    // Create a new HTTP client
    let client = Client::new();

    // Make the GET request to the constructed URL
    let response = match client.get(&url).send() {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("Error making request to {}: {}", url, e);
            return NifStatus::Unknown;
        }
    };

    // Check if the request was successful
    if !response.status().is_success() {
        eprintln!("Request failed with status: {}", response.status());
        return NifStatus::Unknown;
    }

    // Read the response body as text
    let body = match response.text() {
        Ok(text) => text,
        Err(e) => {
            eprintln!("Error reading response body: {}", e);
            return NifStatus::Unknown;
        }
    };

    // Parse the HTML document
    let document = Html::parse_document(&body);

    // Error message selector
    let error_selector = Selector::parse(".alert-message.error.block-message").unwrap();
    if document.select(&error_selector).next().is_some() {
        println!("Found error message for NIF: {}", nif_number);
        return NifStatus::Error;
    }

    // Success message selector
    let success_selector = Selector::parse(".alert-message.success.block-message").unwrap();
    if let Some(success_div) = document.select(&success_selector).next() {
        let text = success_div.text().collect::<String>();
        if text.contains("O NIF indicado é válido mas não conseguimos determinar a entidade associada.") {
            println!("NIF is valid but entity is unknown: {}", nif_number);
            return NifStatus::ValidUnknown;
        } else {
            println!("Found success message for NIF: {}", nif_number);
            // Continue to check for known entity below
        }
    }

    // Multiple results: look for #search-results
    let search_results_selector = Selector::parse("#search-results").unwrap();
    if let Some(search_results) = document.select(&search_results_selector).next() {
        let company_selector = Selector::parse(".search-title").unwrap();
        if search_results.select(&company_selector).next().is_some() {
            println!("Found multiple companies for NIF: {}", nif_number);
            return NifStatus::MultipleResults;
        }
    }

    // Valid and known entity: look for .big-nif and .search-title
    let big_nif_selector = Selector::parse(".big-nif").unwrap();
    let company_selector = Selector::parse(".search-title").unwrap();
    if document.select(&big_nif_selector).next().is_some() &&
       document.select(&company_selector).next().is_some() {
        println!("Found known entity for NIF: {}", nif_number);
        return NifStatus::ValidKnown;
    }

    // If none of the above, check if the page says "NIF não encontrado" or similar
    println!("Could not determine status for NIF: {}", nif_number);
    NifStatus::Unknown
}

/// Validates a Portuguese NIF using only the mathematical algorithm (no external lookup).
pub fn is_nif_valid_local(nif: &str) -> bool {
    // Checks if it has 9 digits
    if nif.len() != 9 || !nif.chars().all(|c| c.is_digit(10)) {
        return false;
    }

    // Checks if the first digit is allowed
    let first = &nif[0..1];
    let first_two = &nif[0..2];
    let valid_first = matches!(
        first,
        "1" | "2" | "3" | "5" | "6" | "7" | "8" | "9"
    ) || first_two == "45";
    if !valid_first {
        return false;
    }

    // Extracts the digits
    let digits: Vec<u32> = nif.chars().map(|c| c.to_digit(10).unwrap()).collect();

    // Calculates the check digit
    let mut sum = 0;
    for (i, d) in digits.iter().take(8).enumerate() {
        sum += d * (9 - i as u32);
    }
    let resto = sum % 11;
    let check_digit = if resto == 0 || resto == 1 { 0 } else { 11 - resto };

    // Compares with the 9th digit
    check_digit == digits[8]
}


/*
    Test on your own with known NIFs or random numbers
    The relevant code is above
*/
fn main() {
    const DEBUG_MODE: u8 = 0; // Set to 0 for CLI mode, 1 for hard-coded NIFs

    if DEBUG_MODE == 1 {
        // Hard-coded NIFs for testing
        let nif_to_check_success = "500960046";
        let nif_to_check_error = "000000001";
        let nif_to_check_multiple = "000000000";

        for nif in &[nif_to_check_success, nif_to_check_error, nif_to_check_multiple] {
            println!("\n--- Checking NIF: {} ---", nif);
            match check_nif_status(nif) {   
                NifStatus::ValidKnown => println!("NIF {} status: Valid and known entity.", nif),
                NifStatus::ValidUnknown => println!("NIF {} status: Valid but unknown entity.", nif),
                NifStatus::Error => println!("NIF {} status: Invalid (Error message).", nif),
                NifStatus::MultipleResults => println!("NIF {} status: Multiple companies found, NIF unavailable.", nif),
                NifStatus::Unknown => println!("NIF {} status: Unknown or could not determine.", nif),
            }
        }

        // Example of local validation (no external lookup)
        let nifs = [
            "123456789", // valid (example from prompt)
            "500829993", // real, probably valid
            "000000001", // invalid
            "987654321", // probably invalid
            "451234567", // starts with 45, allowed
            "012345678", // starts with 0, invalid
        ];
        println!("\n--- Local NIF validation ---");
        for nif in &nifs {
            let valido = is_nif_valid_local(nif);
            println!("NIF {} is {} (local)", nif, if valido { "valid" } else { "invalid" });
        }
    } else {
        // Command line argument mode
        let args: Vec<String> = std::env::args().collect();
        if args.len() < 2 {
            eprintln!("Usage: {} <NIF_NUMBER>", args[0]);
            return;
        }
        let nif_from_args = &args[1];
        println!("\n--- Checking NIF from arguments: {} ---", nif_from_args);
        match check_nif_status(nif_from_args) {
            NifStatus::ValidKnown => println!("NIF {} status: Valid and known entity.", nif_from_args),
            NifStatus::ValidUnknown => println!("NIF {} status: Valid but unknown entity.", nif_from_args),
            NifStatus::Error => println!("NIF {} status: Invalid (Error message).", nif_from_args),
            NifStatus::MultipleResults => println!("NIF {} status: Multiple companies found, NIF unavailable.", nif_from_args),
            NifStatus::Unknown => println!("NIF {} status: Unknown or could not determine.", nif_from_args),
        }
        // Local validation for argument
        let valido = is_nif_valid_local(nif_from_args);
        println!("NIF {} is {} (local)", nif_from_args, if valido { "valid" } else { "invalid" });
    }
}
