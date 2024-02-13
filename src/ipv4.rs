use regex::Regex;

// Avoid initialization of static variable until it is actually needed
lazy_static! {
    static ref REGEX_IPV4: Regex =
        Regex::new(r"(?<quad1>\d+).(?<quad2>\d+).(?<quad3>\d+).(?<quad4>\d+)").unwrap();
}

// This function converts an IPv4 address from its string representation
// to its decimal representation.
pub fn to_decimal(ip: &str) -> u32 {
    let cap = REGEX_IPV4.captures(ip).unwrap();
    let (quad1, quad2, quad3, quad4) = (
        cap["quad1"].parse::<u32>().unwrap(),
        cap["quad2"].parse::<u32>().unwrap(),
        cap["quad3"].parse::<u32>().unwrap(),
        cap["quad4"].parse::<u32>().unwrap(),
    );

    (quad1 << 24) + (quad2 << 16) + (quad3 << 8) + quad4
}

// This function converts a decimal representation of an IPv4 address
// back into its string representation.
pub fn to_ipv4(deci: u32) -> String {
    let b_str = format!("{:032b}", deci);
    let a = &b_str[..1];
    let (quad1, quad2, quad3, quad4) = (&b_str[..8], &b_str[8..16], &b_str[16..24], &b_str[24..]);
    format!(
        "{}.{}.{}.{}",
        u8::from_str_radix(quad1, 2).unwrap(),
        u8::from_str_radix(quad2, 2).unwrap(),
        u8::from_str_radix(quad3, 2).unwrap(),
        u8::from_str_radix(quad4, 2).unwrap()
    )
}

// Applies a netmask to an IPv4 address
pub fn apply_mask(network: &str, netmask: &str) -> u32 {
    let net_deci = to_decimal(network);
    let netmask_deci = to_decimal(netmask);

    net_deci & netmask_deci
}

// This function extracts the significant bits of the network address.
pub fn apply_mask_prefix(network: &str, netmask: &str) -> u32 {
    let not_shift = apply_mask(network, netmask);
    let mask_digit = netmask_digit(netmask);

    not_shift >> (32 - mask_digit)
}

// This function calculates the number of bits set to 1 in the netmask
pub fn netmask_digit(mask: &str) -> i32 {
    let mask_deci = to_decimal(mask);
    let b_str: String = format!("{:032b}", mask_deci);
    b_str.bytes().filter(|&x| x == b'1').count() as i32
}

pub fn netnask_increase(mask: &str) -> String {
    let digits = netmask_digit(mask);
    let mask_deci = to_decimal(mask);

    to_ipv4(mask_deci + (1 << (31 - digits)))
}

pub fn divide_prefix(prefix: &str, mask: &str) -> (String, String) {
    if apply_mask_prefix(prefix, mask) % 2 == 0 {
        let divided_prefix = to_ipv4(to_decimal(prefix) + (1 << (32 - netmask_digit(mask))));
        return (prefix.to_string(), divided_prefix.to_string());
    } 
    
    let divided_prefix = to_ipv4(to_decimal(prefix) - (1 << (32 - netmask_digit(mask)))); 
    (divided_prefix.to_string(), prefix.to_string())
}

// Compare target prefix with the network prefix after applying the netmask.
pub fn check_match(prefix: &str, netmask: &str, network: &str) -> bool {
    let prefix_deci = to_decimal(prefix);
    let net_prefix_deci = apply_mask(network, netmask);

    prefix_deci == net_prefix_deci
}
