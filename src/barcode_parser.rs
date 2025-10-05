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
    pub passenger_status: String,
    pub conditional_data: Option<String>,
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
    let passenger_name = chars[2..22].iter().collect::<String>().trim().to_string();

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
    let token0 = tokens[0];
    let e_ticket_indicator = if !token0.is_empty() {
        token0.chars().next().unwrap_or(' ').to_string()
    } else {
        " ".to_string()
    };
    let booking_code = if token0.len() > 1 {
        token0[1..].to_string()
    } else {
        "".to_string()
    };

    // Token 1: Origin + Destination + Airline (e.g., "CGKSUBGA" = CGK+SUB+GA)
    let token1 = tokens[1];
    if token1.len() < 8 {
        return None;
    }
    let origin = token1[0..3].to_string();
    let destination = token1[3..6].to_string();
    let airline_code = token1[6..8].to_string();

    // Token 2: Flight number (e.g., "0312", "6473", "1900", "6306")
    let flight_number = tokens[2].to_string();

    // Token 3: Julian date + Class + Seat + Sequence (e.g., "260Y045C0120")
    // Format: <julian:3><class:1><seat:4><seq:4><passenger_status:1>
    let token3 = tokens[3];

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
    let seat_number = if token3.len() >= 8 {
        token3[4..8].trim().to_string()
    } else {
        "".to_string()
    };
    let sequence_number = if token3.len() >= 12 {
        token3[8..12].trim().to_string()
    } else {
        "".to_string()
    };
    let passenger_status = if token3.len() > 12 {
        token3.chars().nth(12).unwrap_or('0').to_string()
    } else {
        "0".to_string()
    };

    // Conditional data (everything after token 3)
    let conditional_data = if tokens.len() > 4 {
        Some(tokens[4..].join(" "))
    } else {
        None
    };

    Some(PDF417Data {
        passenger_name: passenger_name.replace('/', " "),
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
        passenger_status,
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

    let passenger_name = chars[2..22].iter().collect::<String>().trim().to_string();
    let e_ticket_indicator = chars[22].to_string();
    let booking_code = chars[23..29].iter().collect::<String>().trim().to_string();
    let origin = chars[29..32].iter().collect::<String>().to_string();
    let destination = chars[32..35].iter().collect::<String>().to_string();
    let airline_code = chars[35..37].iter().collect::<String>().to_string();
    let flight_number = chars[37..42].iter().collect::<String>().trim().to_string();
    let flight_date_julian = chars[42..45].iter().collect::<String>().to_string();
    let cabin_class = chars[45].to_string();
    let seat_number = if chars.len() >= 50 {
        chars[46..50].iter().collect::<String>().trim().to_string()
    } else {
        "".to_string()
    };
    let sequence_number = if chars.len() >= 54 {
        chars[50..54].iter().collect::<String>().trim().to_string()
    } else {
        "".to_string()
    };
    let passenger_status = if chars.len() > 54 {
        chars[54].to_string()
    } else {
        "0".to_string()
    };

    // Conditional data (everything after position 55)
    let conditional_data = if chars.len() > 55 {
        Some(chars[55..].iter().collect::<String>().trim().to_string())
    } else {
        None
    };

    Some(PDF417Data {
        passenger_name: passenger_name.replace('/', " "),
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
        passenger_status,
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
}
