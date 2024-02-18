use regex::Regex;

// Avoid initialization of static variable until it is actually needed
lazy_static! {
    static ref REGEX_IPV4: Regex =
        Regex::new(r"(?<quad1>\d+).(?<quad2>\d+).(?<quad3>\d+).(?<quad4>\d+)").unwrap();
}

/**
 * This function converts an IPv4 address from its string representation
 * to its decimal representation.
 * ip: The IP address in IPv4 format.
 * Return the decimal format of the given IP address
 */ 
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

/**
 * This function converts a decimal representation of an IPv4 address
 * back into its string representation.
 * deci: The decimal format of an IPv4 address.
 * Return a string representing the IPv4 format of the given decimal number.
 */
pub fn to_ipv4(deci: u32) -> String {
    let b_str = format!("{:032b}", deci);
    let (quad1, quad2, quad3, quad4) = (&b_str[..8], &b_str[8..16], &b_str[16..24], &b_str[24..]);
    format!(
        "{}.{}.{}.{}",
        u8::from_str_radix(quad1, 2).unwrap(),
        u8::from_str_radix(quad2, 2).unwrap(),
        u8::from_str_radix(quad3, 2).unwrap(),
        u8::from_str_radix(quad4, 2).unwrap()
    )
}

/**
 * This function is used to calculate the deciaml representation
 * of the given IP address after applying the given mask.
 * network: An IPv4 address (net prefix).
 * netmask: An netmask in IPv4 format.
 * Return the decimal representation of the given IPv4 address
 * after applying the mask.
 */
pub fn apply_mask(network: &str, netmask: &str) -> u32 {
    let net_deci = to_decimal(network);
    let netmask_deci = to_decimal(netmask);

    net_deci & netmask_deci
}

/**
 * This function is used to calculate the deciaml representation (only the significant bits part)
 * of the given IP address after applying the given mask.
 * network: An IPv4 address (net prefix).
 * netmask: An netmask in IPv4 format.
 * Return the decimal representation (significant bits) of the given IPv4 address
 * after applying the mask.
 */
pub fn apply_mask_prefix(network: &str, netmask: &str) -> u32 {
    let not_shift = apply_mask(network, netmask);
    let mask_digit = netmask_digit(netmask);

    not_shift >> (32 - mask_digit)
}

/**
 * This function is used to transform the netmask from IPv4 format
 * to digit format.
 * This function assumes that the given subnet mask is valid.
 * i.g. 255.255.0.0 => 16
 * mask: A subnet mask in IPv4 format.
 * Return the digit format of an IPv4 subnet mask.
 */
pub fn netmask_digit(mask: &str) -> i32 {
    let mask_deci = to_decimal(mask);
    let b_str: String = format!("{:032b}", mask_deci);
    b_str.bytes().filter(|&x| x == b'1').count() as i32
}

/**
 * This function transforms a given subnet mask in IPv4 format to another subnet mask in IPv4 format 
 * whose digit format is increased by one.
 * i.g. 255.255.0.0 => 255.255.128.0
 * mask: A subnet mask in IPv4 format.
 * Return a subnet mask in IPv4 format whose digit format is increased by one.
 */
pub fn netnask_increase(mask: &str) -> String {
    let digits = netmask_digit(mask);
    let mask_deci = to_decimal(mask);

    to_ipv4(mask_deci + (1 << (31 - digits)))
}

/**
 * This function divides the given network prefix into two sub-prefixes 
 * of the same size using the provided netmask.
 * i.g. prefix: 192.168.128.0, mask: 255.255.224.0 => (192.168.128.0, 192.168.160.0)
 * prefix: An IPv4 address to be divided.
 * mask: A subnet mask which is used to divide the prefix.
 * Return a tuple of the divided sub-prefixes.
 */
pub fn divide_prefix(prefix: &str, mask: &str) -> (String, String) {
    if apply_mask_prefix(prefix, mask) % 2 == 0 {
        let divided_prefix = to_ipv4(to_decimal(prefix) + (1 << (32 - netmask_digit(mask))));
        return (prefix.to_string(), divided_prefix.to_string());
    } 
    
    let divided_prefix = to_ipv4(to_decimal(prefix) - (1 << (32 - netmask_digit(mask)))); 
    (divided_prefix.to_string(), prefix.to_string())
}

// Compare target prefix with the network prefix after applying the netmask.
/**
 * This function is used to check if the given IP address match the given network prefix
 * after applying the net mask.
 * prefix: A network prefix.
 * netmask: A subnet mask.
 * network: An IPv4 address.
 * Return true if the given address matches the prefix after applying
 * the subnet mask. Otherwise, false.
 */
pub fn check_match(prefix: &str, netmask: &str, network: &str) -> bool {
    let prefix_deci = to_decimal(prefix);
    let net_prefix_deci = apply_mask(network, netmask);

    prefix_deci == net_prefix_deci
}
