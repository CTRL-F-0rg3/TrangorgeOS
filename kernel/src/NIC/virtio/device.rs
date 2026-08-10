pub trait NetworkDevice {
    fn mac_adress(&self) -> MacAdress;

    fn mtu(&self) -> unsize;

    fn capabilitiers(&self) -> Capabilitiers;

    fn transmit(&mut self, packet: &[u8]) -> Result<(), NetworkError>;
    fn receive(&mut self) -> Option<&[u8]>;
}