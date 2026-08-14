// per other programers pls do not delete it beacuse it example for bunker forge it not be us code too implamate 
// thanks my friend {: 

fn a(data: &[u8], start_bit: usize, bit_count: usize) -> u64 {
    let mut value = 0u64;

    for i in 0..bit_count {
        let bit_position = start_bit + i;
        let byte_index = bit_position / 8;
        let bit_index = bit_position % 8;
        let bit = (data[byte_index] >> bit_index) & 1;
        value |= (bit as u64) << i;
    }
    value
}

fn main() {
    let data = [
        0b1011_0101,
        0b1100_001,
        0b0110_1001,
    
    ];

    let value = a(&data, 3, 5);

    println!("format: {}", value);

    let okey = Some(value);

    println!(" OK: {:?}", okey);
}