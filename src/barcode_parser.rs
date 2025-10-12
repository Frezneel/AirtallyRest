// Shared IATA BCBP Parser Module
// This module is SYNCHRONIZED with mobile app (rust/src/api/barcode.rs)
// Any changes here MUST be replicated in mobile app parser!

/// Normalize and clean barcode data - removes control characters but keeps internal spaces
pub fn normalize_barcode_data(raw_data: &str) -> String {
    raw_data
        .replace(['\n', '\r', '\t'], "")
        .chars()
        .filter(|c| c.is_ascii() && (!c.is_control() || *c == ' '))
        .collect()
}

/// PDF417 parsed data structure
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PDF417Data {
    pub passenger_name: String,
    pub e_ticket_indicator: String,
    pub booking_code: String,
    pub origin: String,
    pub destination: String,
    pub airline_code: String,
    pub flight_number: String,
    pub flight_date_julian: String,
    pub cabin_class: String,
    pub seat_number: String,
    pub sequence_number: String,
    pub infant_status: bool,
    pub conditional_data: Option<String>,
}

/// Convert UPPERCASE to Title Case
/// Example: "JOHN SMITH" -> "John Smith"
fn title_case(s: &str) -> String {
    s.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => {
                    let mut result = String::new();
                    result.push_str(&first.to_uppercase().to_string());
                    result.push_str(&chars.as_str().to_lowercase());
                    result
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Format passenger name from IATA format to readable format
/// Input: "LASTNAME/FIRSTNAME TITLE" (e.g., "PUTRI/SITI MS")
/// Output: "Title Firstname Lastname" (e.g., "Ms Siti Putri")
fn format_passenger_name(raw_name: &str) -> String {
    let parts: Vec<&str> = raw_name.split('/').collect();
    if parts.len() != 2 {
        // No slash, return as-is with title case
        return title_case(raw_name);
    }

    let lastname = parts[0].trim();
    let firstname_with_title = parts[1].trim();

    // Common titles to extract
    let titles = ["MR", "MS", "MRS", "MISS", "DR", "PROF"];

    // Check if last word is a title
    let tokens: Vec<&str> = firstname_with_title.split_whitespace().collect();
    let (firstname, title) = if tokens.len() > 1 {
        let last_token = tokens[tokens.len() - 1].to_uppercase();
        if titles.contains(&last_token.as_str()) {
            // Title found
            let first = tokens[..tokens.len() - 1].join(" ");
            (first, Some(last_token))
        } else {
            (firstname_with_title.to_string(), None)
        }
    } else {
        (firstname_with_title.to_string(), None)
    };

    // Format output
    let formatted_firstname = title_case(&firstname);
    let formatted_lastname = title_case(lastname);

    match title {
        Some(t) => {
            let formatted_title = title_case(&t);
            format!("{} {} {}", formatted_title, formatted_firstname, formatted_lastname)
        }
        None => format!("{} {}", formatted_firstname, formatted_lastname),
    }
}

/// Multi-strategy IATA BCBP parser with fallback
/// Synchronized with mobile app parser
pub fn parse_iata_bcbp(barcode: &str) -> Option<PDF417Data> {
    // Normalize first - remove control characters but keep spaces
    let normalized = normalize_barcode_data(barcode);

    if normalized.len() < 50 {
        return None;
    }

    let chars: Vec<char> = normalized.chars().collect();

    if chars.len() < 50 || chars[0] != 'M' {
        return None;
    }

    // Strategy 1: Try space-delimited format (Indonesian airlines)
    if let Some(data) = try_parse_space_delimited(&chars) {
        return Some(data);
    }

    // Strategy 2: Try strict IATA fixed-length format (International airlines)
    if let Some(data) = try_parse_strict_iata(&chars) {
        return Some(data);
    }

    None
}

// Strategy 1: Space-delimited parser (for Indonesian airlines: Garuda, Lion Air, Citilink, Batik Air, AirAsia)
// Format: M1PASSENGER/NAME <spaces> EBOOKING CGKSUBGA <flight> <julian>Y<seat><seq> <extra>
fn try_parse_space_delimited(chars: &[char]) -> Option<PDF417Data> {
    // Extract fixed-position fields (strictly positioned)
    // Passenger name is EXACTLY positions 2-22 (20 chars), trim AFTER extraction
    let passenger_name_raw: String = chars[2..22].iter().collect();
    let passenger_name = passenger_name_raw.trim().to_string();

    // Find the rest after passenger name by splitting on spaces
    let remainder = if chars.len() > 22 {
        chars[22..].iter().collect::<String>()
    } else {
        return None;
    };

    // Split by spaces and extract tokens
    let tokens: Vec<&str> = remainder.split_whitespace().collect();

    if tokens.len() < 4 {
        return None;
    }

    // Token 0: E-ticket indicator + Booking code (e.g., "EE6UVIL", "ESMMTHQ", "ZKMR9K")
    // Special case: If token0 is single VALID e-ticket indicator letter, merge with next token
    // Valid e-ticket indicators: E (Electronic), M (Mobile), Z, T, etc.
    let valid_eticket_indicators = ['E', 'M', 'Z', 'T', 'B'];
    let (e_ticket_indicator, booking_code, token_offset) = if tokens[0].len() == 1 && tokens.len() >= 5 {
        let first_char = tokens[0].chars().next().unwrap().to_uppercase().next().unwrap();
        if valid_eticket_indicators.contains(&first_char) {
            // Valid single e-ticket indicator - merge with next token
            // "E" + "FGH345" = "EFGH345"
            let merged = format!("{}{}", tokens[0], tokens.get(1).unwrap_or(&""));
            let e_ticket = merged.chars().next().unwrap_or(' ').to_string();
            let booking = if merged.len() > 1 {
                merged[1..].to_string()
            } else {
                "".to_string()
            };
            // Shift tokens by 1 (since we merged token 0 and 1)
            (e_ticket, booking, 1)
        } else {
            // Not a valid e-ticket indicator - treat as normal token
            let e_ticket = tokens[0].chars().next().unwrap_or(' ').to_string();
            let booking = if tokens[0].len() > 1 {
                tokens[0][1..].to_string()
            } else {
                "".to_string()
            };
            (e_ticket, booking, 0)
        }
    } else {
        // Normal case: "EFGH345" or multi-char token
        let e_ticket = tokens[0].chars().next().unwrap_or(' ').to_string();
        let booking = if tokens[0].len() > 1 {
            tokens[0][1..].to_string()
        } else {
            "".to_string()
        };
        (e_ticket, booking, 0)
    };

    // Adjust token indices based on offset
    let origin_dest_airline_idx = 1 + token_offset;
    let flight_number_idx = 2 + token_offset;
    let date_class_seat_idx = 3 + token_offset;

    if tokens.len() <= date_class_seat_idx {
        return None;
    }

    // Token 1 (or 2): Origin + Destination + Airline (e.g., "CGKSUBGA" = CGK+SUB+GA)
    let token1 = tokens[origin_dest_airline_idx];
    if token1.len() < 8 {
        return None;
    }
    let origin = token1[0..3].to_string();
    let destination = token1[3..6].to_string();
    let airline_code = token1[6..8].to_string();

    // Token 2 (or 3): Flight number (e.g., "0312", "6473", "1900", "6306")
    let flight_number = tokens[flight_number_idx].to_string();

    // Token 3 (or 4): Julian date + Class + Seat + Sequence (e.g., "260Y045C0120")
    // Format: <julian:3><class:1><seat:4><seq:4><passenger_status:1>
    let token3 = tokens[date_class_seat_idx];

    let flight_date_julian = if token3.len() >= 3 {
        token3[0..3].to_string()
    } else {
        return None;
    };
    let cabin_class = if token3.len() >= 4 {
        token3.chars().nth(3).unwrap_or('Y').to_string()
    } else {
        "Y".to_string()
    };
    let seat_number_raw = if token3.len() >= 8 {
        token3[4..8].trim().to_string()
    } else {
        "".to_string()
    };
    let sequence_number = if token3.len() >= 12 {
        token3[8..12].trim().to_string()
    } else {
        "".to_string()
    };

    // Detect infant passenger by checking seat number
    let infant_status = seat_number_raw.contains("INF");
    let seat_number = if infant_status {
        "".to_string() // Infants don't have seat assignments
    } else {
        seat_number_raw
    };

    // Conditional data (everything after the date/class/seat token)
    let conditional_data_idx = date_class_seat_idx + 1;
    let conditional_data = if tokens.len() > conditional_data_idx {
        Some(tokens[conditional_data_idx..].join(" "))
    } else {
        None
    };

    Some(PDF417Data {
        passenger_name: format_passenger_name(&passenger_name),
        e_ticket_indicator,
        booking_code,
        origin,
        destination,
        airline_code,
        flight_number,
        flight_date_julian,
        cabin_class,
        seat_number,
        sequence_number,
        infant_status,
        conditional_data,
    })
}

// Strategy 2: Strict IATA fixed-length parser (for international airlines)
// Format: M1NAME(20)E(1)BOOKING(6)ORIGIN(3)DEST(3)AIRLINE(2)FLIGHT(5)JULIAN(3)CLASS(1)SEAT(4)SEQ(4)STATUS(1)
fn try_parse_strict_iata(chars: &[char]) -> Option<PDF417Data> {
    // Minimum length for strict IATA: 2 + 20 + 1 + 6 + 3 + 3 + 2 + 5 + 3 + 1 + 4 + 4 + 1 = 55
    if chars.len() < 55 {
        return None;
    }

    // Check if there are NOT many spaces (indicating strict format)
    let space_count = chars.iter().filter(|&&c| c == ' ').count();
    if space_count > 5 {
        // Too many spaces, likely not strict IATA format
        return None;
    }

    // IMPORTANT: Don't trim before slicing - positions are fixed!
    // Passenger name is EXACTLY positions 2-22 (20 chars), trim AFTER extraction
    let passenger_name_raw: String = chars[2..22].iter().collect();
    let passenger_name = passenger_name_raw.trim().to_string();

    let e_ticket_indicator = chars[22].to_string();
    let booking_code = chars[23..29].iter().collect::<String>().trim().to_string();
    let origin = chars[29..32].iter().collect::<String>().to_string();
    let destination = chars[32..35].iter().collect::<String>().to_string();
    let airline_code = chars[35..37].iter().collect::<String>().to_string();
    let flight_number = chars[37..42].iter().collect::<String>().trim().to_string();
    let flight_date_julian = chars[42..45].iter().collect::<String>().to_string();
    let cabin_class = chars[45].to_string();
    let seat_number_raw = if chars.len() >= 50 {
        chars[46..50].iter().collect::<String>().trim().to_string()
    } else {
        "".to_string()
    };
    let sequence_number = if chars.len() >= 54 {
        chars[50..54].iter().collect::<String>().trim().to_string()
    } else {
        "".to_string()
    };

    // Detect infant passenger by checking seat number
    let infant_status = seat_number_raw.contains("INF");
    let seat_number = if infant_status {
        "".to_string() // Infants don't have seat assignments
    } else {
        seat_number_raw
    };

    // Conditional data (everything after position 55)
    let conditional_data = if chars.len() > 55 {
        Some(chars[55..].iter().collect::<String>().trim().to_string())
    } else {
        None
    };

    Some(PDF417Data {
        passenger_name: format_passenger_name(&passenger_name),
        e_ticket_indicator,
        booking_code,
        origin,
        destination,
        airline_code,
        flight_number,
        flight_date_julian,
        cabin_class,
        seat_number,
        sequence_number,
        infant_status,
        conditional_data,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_garuda() {
        let barcode = "M1PRASETYO/YUDHA DWI  EE6UVIL CGKSUBGA 0312 260Y045C0120 348>5180  5259B1A              2A12621429493830 GA                        N";
        let parsed = parse_iata_bcbp(barcode);
        assert!(parsed.is_some());
        let data = parsed.unwrap();
        assert_eq!(data.airline_code, "GA");
        assert_eq!(data.flight_date_julian, "260");
    }

    #[test]
    fn test_parse_lion_air() {
        let barcode = "M1BAYU/MUHAMMAD MR    ESMMTHQ DHXCGKID 6473 032Y007A0002 300.";
        let parsed = parse_iata_bcbp(barcode);
        assert!(parsed.is_some());
        let data = parsed.unwrap();
        assert_eq!(data.airline_code, "ID");
        assert_eq!(data.flight_date_julian, "032");
    }

    #[test]
    fn test_parse_citilink() {
        let barcode = "M1LADOA/RICKYFEBRIANTO ZKMR9K SUBCGKQG 0725 168Y017A0016 147>1181WW5166BQG 000000000000029177000000000- 0";
        let parsed = parse_iata_bcbp(barcode);
        assert!(parsed.is_some());
        let data = parsed.unwrap();
        assert_eq!(data.airline_code, "QG");
        assert_eq!(data.flight_date_julian, "168");
    }

    #[test]
    fn test_parse_batik_air() {
        let barcode = "M1ABU TALIB/SUZANA MS EQQZBWR KULTWUOD 1900 129Y012F0118 100";
        let parsed = parse_iata_bcbp(barcode);
        assert!(parsed.is_some());
        let data = parsed.unwrap();
        assert_eq!(data.airline_code, "OD");
        assert_eq!(data.flight_date_julian, "129");
    }

    #[test]
    fn test_parse_airasia() {
        let barcode = "M1Ongere/Mark Mokaya  EPBC4GN KULLGKAK 6306 108Y019B0026 11E>3180MM    B                00";
        let parsed = parse_iata_bcbp(barcode);
        assert!(parsed.is_some());
        let data = parsed.unwrap();
        assert_eq!(data.airline_code, "AK");
        assert_eq!(data.flight_date_julian, "108");
    }

    #[test]
    fn test_parse_infant_ticket() {
        // Infant ticket - Real barcode with INF in seat field
        let barcode = "M1MAYZURA/AUFARIZA HANEBJQUJW CGKUPGID 6296 147Y0INF0097 100";
        let parsed = parse_iata_bcbp(barcode);
        assert!(parsed.is_some());
        let data = parsed.unwrap();
        assert_eq!(data.passenger_name, "Aufariza Han Mayzura"); // Formatted name (limited to 20 chars raw)
        assert_eq!(data.e_ticket_indicator, "E");
        assert_eq!(data.booking_code, "BJQUJW");
        assert_eq!(data.origin, "CGK");
        assert_eq!(data.destination, "UPG");
        assert_eq!(data.airline_code, "ID");
        assert_eq!(data.flight_number, "6296");
        assert_eq!(data.flight_date_julian, "147");
        assert_eq!(data.cabin_class, "Y");
        assert_eq!(data.seat_number, ""); // Infants have no seat
        assert_eq!(data.sequence_number, "0097");
        assert_eq!(data.infant_status, true); // Infant status
    }

    #[test]
    fn test_parse_non_infant_ticket() {
        // Regular ticket - should have infant_status = false
        let barcode = "M1PRASETYO/YUDHA DWI  EE6UVIL CGKSUBGA 0312 260Y045C0120 348>5180  5259B1A              2A12621429493830 GA                        N";
        let parsed = parse_iata_bcbp(barcode);
        assert!(parsed.is_some());
        let data = parsed.unwrap();
        assert_eq!(data.infant_status, false);
        assert_eq!(data.seat_number, "045C");
    }

    #[test]
    fn test_name_formatting_with_title() {
        // User's requested format: PUTRI/SITI MS -> Ms Siti Putri
        let barcode = "M1PUTRI/SITI MS       EXYZ789 CGKSUBJT 0610 277Y023B0045 300";
        let parsed = parse_iata_bcbp(barcode);
        assert!(parsed.is_some());
        let data = parsed.unwrap();
        assert_eq!(data.passenger_name, "Ms Siti Putri");
        assert_eq!(data.e_ticket_indicator, "E"); // Ticket type
        assert_eq!(data.booking_code, "XYZ789"); // Booking code without E
        assert_eq!(data.origin, "CGK");
        assert_eq!(data.destination, "SUB");
        assert_eq!(data.airline_code, "JT");
        assert_eq!(data.flight_number, "0610");
        assert_eq!(data.flight_date_julian, "277");
    }

    #[test]
    fn test_name_formatting_mr_title() {
        // BAYU/MUHAMMAD MR -> Mr Muhammad Bayu
        let barcode = "M1BAYU/MUHAMMAD MR    ESMMTHQ DHXCGKID 6473 032Y007A0002 300.";
        let parsed = parse_iata_bcbp(barcode);
        assert!(parsed.is_some());
        let data = parsed.unwrap();
        assert_eq!(data.passenger_name, "Mr Muhammad Bayu");
    }

    #[test]
    fn test_name_formatting_compound_lastname() {
        // ABU TALIB/SUZANA MS -> Ms Suzana Abu Talib
        let barcode = "M1ABU TALIB/SUZANA MS EQQZBWR KULTWUOD 1900 129Y012F0118 100";
        let parsed = parse_iata_bcbp(barcode);
        assert!(parsed.is_some());
        let data = parsed.unwrap();
        assert_eq!(data.passenger_name, "Ms Suzana Abu Talib");
    }

    #[test]
    fn test_name_formatting_no_title() {
        // SMITH/JOHN -> John Smith (no title)
        let barcode = "M1SMITH/JOHN          EABC123 CGKJKTGA 0001 001Y001A0001 100";
        let parsed = parse_iata_bcbp(barcode);
        assert!(parsed.is_some());
        let data = parsed.unwrap();
        assert_eq!(data.passenger_name, "John Smith");
    }

    #[test]
    fn test_short_name_no_e_in_passenger_name() {
        // Bug fix: Short names should not include E-ticket indicator
        // "AMELIA/VINO" (11 chars) + 9 spaces = 20 chars total for name field
        // Position 2-21 (20 chars): "AMELIA/VINO         "
        // Position 22: " " (space before E)
        // Position 23: "E" (e-ticket indicator)
        let barcode = "M1AMELIA/VINO         EFGH345 CGKBDOQG 1630 284Y029A0045 290>4012WC0011BQG 000000000000056789000000000- 0";
        let parsed = parse_iata_bcbp(barcode);
        assert!(parsed.is_some());
        let data = parsed.unwrap();
        // Passenger name should NOT include the "E"
        assert_eq!(data.passenger_name, "Vino Amelia");
        assert_eq!(data.e_ticket_indicator, "E");
        assert_eq!(data.booking_code, "FGH345");
        assert_eq!(data.origin, "CGK");
        assert_eq!(data.destination, "BDO");
        assert_eq!(data.airline_code, "QG");
        assert_eq!(data.flight_number, "1630");
        assert_eq!(data.flight_date_julian, "284");
    }

    #[test]
    fn test_booking_code_starting_with_g() {
        // Bug fix: Booking code starting with "G" should NOT merge with name
        // "OKTAVIA/KENNY" (13 chars) + 7 spaces = 20 chars total for name field
        // "G" is NOT a valid e-ticket indicator, so should not be merged
        let barcode = "M1OKTAVIA/KENNY       GHIJ567 CGKBDOQG 1630 284Y002O0012 334>8457BX8890BQG 000000000000062747000000000- 0";
        let parsed = parse_iata_bcbp(barcode);
        assert!(parsed.is_some());
        let data = parsed.unwrap();
        // Passenger name should NOT include "G"
        assert_eq!(data.passenger_name, "Kenny Oktavia");
        assert_eq!(data.e_ticket_indicator, "G");
        assert_eq!(data.booking_code, "HIJ567");
        assert_eq!(data.origin, "CGK");
        assert_eq!(data.destination, "BDO");
        assert_eq!(data.airline_code, "QG");
        assert_eq!(data.flight_number, "1630");
        assert_eq!(data.flight_date_julian, "284");
    }
}
