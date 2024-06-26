// Server that displays IO Status

use axum::{
    extract::State,
    routing::get,
    Router,
    response::Html,
};


use std::sync::{Arc, Mutex};
use std::time::Duration;
use rppal::gpio::Gpio;
use std::io;
use std::any::type_name;
use std::error::Error;
use std::thread;
use rppal::pwm::{Channel, Polarity, Pwm};
use rppal::i2c::I2c;

// ADS1115 I2C address when ADDR pin pulled to ground
const ADDR_ADS115:     u16 = 0x48; // Address of first ADS115 chip  
const ADDR_ADS115_TWO: u16 = 0x49; // Address of second ADS115 chip

// ADS115 register addresses.
const REG_CONFIGURATION: u8 = 0x01;
const REG_CONVERSION:    u8 = 0x00;
const DELAY_TIME:        u64 = 200; 
const VOLTAGE_LIMIT:     f32 = 6.5; 



async fn hello_world() -> String {
    let a = 40;
    format!("Hello, World! i am {}", a)
}

// const IO_PAGE: &str = ;

async fn get_io_status(State(state): State<Arc<Mutex<IoState>>>) -> Html<String> {
    let io_state = state.lock().unwrap();

    Html(format!(r#"
    <html>
    <title>Io Page</title>
    <body>
    Pin 24: {} Pin 25: {}
    <script>
    setTimeout(function() {{
        document.location.reload(true);
    }}, 50);
    </script>
    </body>
    "#, io_state.pin_one, io_state.pin_two))
}

struct IoState {
    // digital input pins.
    pin_one: bool,
    pin_two: bool,
    pin_three: bool, 
}

impl IoState {
    fn new() -> Self {
        Self {
            pin_one: false,
            pin_two: false,
            pin_three: false,
        }
    }

    fn set_pin(&mut self, pin: u8, value: bool) {
        if pin == 0{ 
            self.pin_one = value;
        }
        else if pin == 1 { 
            self.pin_two = value;
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    let shared_state: Arc<Mutex<IoState>> = Arc::new(Mutex::new(IoState::new()));
    let background_state = shared_state.clone();

    let mut pin_pic_one = String::new();
    println!("Pick the first Raspberry PI output pin number"); //PIN 20 is connected for pin pick
    io::stdin().read_line(&mut pin_pic_one).expect("Failed to read line");

    let mut pick_one = get_pin_number(pin_pic_one);
    println!("pick_one equals: {pick_one}");

    let mut i2c = I2c::new()?;
    i2c.set_slave_address(ADDR_ADS115)?; // Set the I2C slave address to the device we're communicating with.

    i2c.block_write(REG_CONFIGURATION, &[0x42, 0x82],)?; // Set configuration setting to ADS115
    thread::sleep(Duration::from_millis(DELAY_TIME));

    i2c.block_write(REG_CONVERSION, &[0x00],)?;       // Set ADS115 config to look at the conversion registers 
    thread::sleep(Duration::from_millis(DELAY_TIME));

    let mut reg = [0u8; 2];

    i2c.block_read(0x00, &mut reg)?;       // reads ADS115 conversion register and puts contents into reg buffer
    thread::sleep(Duration::from_millis(DELAY_TIME));


    let mut adc0val:u16 = u16::from_be_bytes(reg);
    println!(" ADC 0 decimal value = {:?} ", adc0val);
    let mut adc0voltage:f32 = adc0val.into(); 

    let mut adc0voltage:f32 = adc0voltage * 0.000125;
    println!(" ADC 0 voltage = {:?} ", adc0voltage);


    tokio::spawn(async move {
        // set input pins
        let mut pin_25 = Gpio::new().unwrap().get(25).unwrap().into_input_pulldown();
        let mut pin_24 = Gpio::new().unwrap().get(24).unwrap().into_input_pulldown();
        // set the user selected outputs 
        let mut pin_selection_one = Gpio::new().unwrap().get(pick_one).unwrap().into_output();

    
        

        loop {
            {
                let mut io_state = background_state.lock().unwrap();
                io_state.set_pin(0, pin_24.is_high());
                io_state.set_pin(1, pin_25.is_high());

                pin_selection_one.toggle();
                thread::sleep(Duration::from_millis(250));

                get_adc0_value();
                get_adc1_value();
                get_adc2_value();
                get_adc3_value();
                println!("");
                get_adc0_2_value();
                get_adc1_2_value();
                get_adc2_2_value();
                get_adc3_2_value();
                println!("");

            
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
            // sleep for half a second
            // toggle pin values
            // sleep a bit 
            // toggle
        }

        
    });

    // build our application with a single route
    let app = Router::new()
                    .route("/", get(hello_world))
                    .route("/io", get(get_io_status))
                    .with_state(shared_state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

fn get_pin_number(x: String) -> u8 {

    let mut num = x.trim();
    let mut num = num.parse::<u8>().unwrap();
    println!("num equals: {num}");
    
    num   
}


fn get_adc0_value() -> Result<(), Box<dyn Error>>
{

    let mut adc0_reg = [0u8; 2];

    let mut i2c0 = I2c::new()?;
    i2c0.set_slave_address(ADDR_ADS115)?;

    i2c0.block_write(REG_CONFIGURATION, &[0x42, 0x82],)?; // Set configuration setting to ADS115
    thread::sleep(Duration::from_millis(DELAY_TIME));

    i2c0.block_write(REG_CONVERSION, &[0x00],)?;       // Set ADS115 config to look at the conversion registers 
    thread::sleep(Duration::from_millis(DELAY_TIME));

    i2c0.block_read(REG_CONVERSION, &mut adc0_reg)?;       // reads ADS115 conversion register and puts contents into reg buffer
    thread::sleep(Duration::from_millis(DELAY_TIME));


    let mut adc0val:u16 = u16::from_be_bytes(adc0_reg);
    //println!(" ADC 0 decimal value = {:?} ", adc0val);
    let mut adc0voltage:f32 = adc0val.into(); 

    let mut adc0voltage:f32 = adc0voltage * 0.000125;
    if adc0voltage > VOLTAGE_LIMIT {
         adc0voltage = 0.01;
    }
    println!(" ADC_1 0 voltage = {:?} ", adc0voltage);
    
    Ok(())

}
    

fn get_adc1_value() -> Result<(), Box<dyn Error>>
{

    let mut adc1_reg = [0u8; 2];

    let mut i2c1 = I2c::new()?;
    i2c1.set_slave_address(ADDR_ADS115)?;

    i2c1.block_write(REG_CONFIGURATION, &[0x52, 0x82],)?; // Set configuration setting to ADS115
    thread::sleep(Duration::from_millis(DELAY_TIME));

    i2c1.block_write(REG_CONVERSION, &[0x00],)?;       // Set ADS115 config to look at the conversion registers 
    thread::sleep(Duration::from_millis(DELAY_TIME));

    i2c1.block_read(REG_CONVERSION, &mut adc1_reg)?;       // reads ADS115 conversion register and puts contents into reg buffer
    thread::sleep(Duration::from_millis(DELAY_TIME));


    let mut adc1val:u16 = u16::from_be_bytes(adc1_reg);
    //println!(" ADC 1 decimal value = {:?} ", adc1val);
    let mut adc1voltage:f32 = adc1val.into(); 

    let mut adc1voltage:f32 = adc1voltage * 0.000125;
    if adc1voltage > VOLTAGE_LIMIT {
        adc1voltage = 0.01;
   }
    println!(" ADC_1 1 voltage = {:?} ", adc1voltage);
    
    Ok(())

}




fn get_adc2_value() -> Result<(), Box<dyn Error>>
{

    let mut adc2_reg = [0u8; 2];

    let mut i2c2 = I2c::new()?;
    i2c2.set_slave_address(ADDR_ADS115)?;

    i2c2.block_write(REG_CONFIGURATION, &[0x62, 0x82],)?; // Set configuration setting to ADS115
    thread::sleep(Duration::from_millis(DELAY_TIME));

    i2c2.block_write(REG_CONVERSION, &[0x00],)?;       // Set ADS115 config to look at the conversion registers 
    thread::sleep(Duration::from_millis(DELAY_TIME));

    i2c2.block_read(REG_CONVERSION, &mut adc2_reg)?;       // reads ADS115 conversion register and puts contents into reg buffer
    thread::sleep(Duration::from_millis(DELAY_TIME));


    let mut adc2val:u16 = u16::from_be_bytes(adc2_reg);
    //println!(" ADC 2 decimal value = {:?} ", adc2val);
    let mut adc2voltage:f32 = adc2val.into(); 

    let mut adc2voltage:f32 = adc2voltage * 0.000125;
    if adc2voltage > VOLTAGE_LIMIT {
        adc2voltage = 0.01;
   }
    println!(" ADC_1 2 voltage = {:?} ", adc2voltage);
    
    Ok(())

}


fn get_adc3_value() -> Result<(), Box<dyn Error>>
{

    let mut adc3_reg = [0u8; 2];

    let mut i2c3 = I2c::new()?;
    i2c3.set_slave_address(ADDR_ADS115)?;

    i2c3.block_write(REG_CONFIGURATION, &[0x72, 0x82],)?; // Set configuration setting to ADS115
    thread::sleep(Duration::from_millis(DELAY_TIME));

    i2c3.block_write(REG_CONVERSION, &[0x00],)?;       // Set ADS115 config to look at the conversion registers 
    thread::sleep(Duration::from_millis(DELAY_TIME));

    i2c3.block_read(REG_CONVERSION, &mut adc3_reg)?;       // reads ADS115 conversion register and puts contents into reg buffer
    thread::sleep(Duration::from_millis(DELAY_TIME));


    let mut adc3val:u16 = u16::from_be_bytes(adc3_reg);
    //println!(" ADC 3 decimal value = {:?} ", adc3val);
    let mut adc3voltage:f32 = adc3val.into(); 

    let mut adc3voltage:f32 = adc3voltage * 0.000125;
    if adc3voltage > VOLTAGE_LIMIT {
        adc3voltage = 0.01;
   }
    println!(" ADC_1 3 voltage = {:?} ", adc3voltage);
    
    Ok(())

}


fn get_adc0_2_value() -> Result<(), Box<dyn Error>>  // this is a second ADS1115 ADC slave chip
{

    let mut adc0_2_reg = [0u8; 2];

    let mut i2c0 = I2c::new()?;
    i2c0.set_slave_address(ADDR_ADS115_TWO)?;

    i2c0.block_write(REG_CONFIGURATION, &[0x42, 0x82],)?; // Set configuration setting to ADS115
    thread::sleep(Duration::from_millis(DELAY_TIME));

    i2c0.block_write(REG_CONVERSION, &[0x00],)?;       // Set ADS115 config to look at the conversion registers 
    thread::sleep(Duration::from_millis(DELAY_TIME));

    i2c0.block_read(REG_CONVERSION, &mut adc0_2_reg)?;       // reads ADS115 conversion register and puts contents into reg buffer
    thread::sleep(Duration::from_millis(DELAY_TIME));


    let mut adc0_2_val:u16 = u16::from_be_bytes(adc0_2_reg);
    //println!(" ADC 0 decimal value = {:?} ", adc0val);
    let mut adc0_2_voltage:f32 = adc0_2_val.into(); 

    let mut adc0_2_voltage:f32 = adc0_2_voltage * 0.000125;
    println!(" ADC_2 0 voltage = {:?} ", adc0_2_voltage);
    
    Ok(())

}


fn get_adc1_2_value() -> Result<(), Box<dyn Error>>  // this is a second ADS1115 ADC slave chip
{

    let mut adc1_2_reg = [0u8; 2];

    let mut i2c0 = I2c::new()?;
    i2c0.set_slave_address(ADDR_ADS115_TWO)?;

    i2c0.block_write(REG_CONFIGURATION, &[0x52, 0x82],)?; // Set configuration setting to ADS115
    thread::sleep(Duration::from_millis(DELAY_TIME));

    i2c0.block_write(REG_CONVERSION, &[0x00],)?;       // Set ADS115 config to look at the conversion registers 
    thread::sleep(Duration::from_millis(DELAY_TIME));

    i2c0.block_read(REG_CONVERSION, &mut adc1_2_reg)?;       // reads ADS115 conversion register and puts contents into reg buffer
    thread::sleep(Duration::from_millis(DELAY_TIME));


    let mut adc1_2_val:u16 = u16::from_be_bytes(adc1_2_reg);
    //println!(" ADC 0 decimal value = {:?} ", adc0val);
    let mut adc1_2_voltage:f32 = adc1_2_val.into(); 

    let mut adc1_2_voltage:f32 = adc1_2_voltage * 0.000125;
    println!(" ADC_2 1 voltage = {:?} ", adc1_2_voltage);
    
    Ok(())

}


fn get_adc2_2_value() -> Result<(), Box<dyn Error>>  // this is a second ADS1115 ADC slave chip
{

    let mut adc2_2_reg = [0u8; 2];

    let mut i2c0 = I2c::new()?;
    i2c0.set_slave_address(ADDR_ADS115_TWO)?;

    i2c0.block_write(REG_CONFIGURATION, &[0x62, 0x82],)?; // Set configuration setting to ADS115
    thread::sleep(Duration::from_millis(DELAY_TIME));

    i2c0.block_write(REG_CONVERSION, &[0x00],)?;       // Set ADS115 config to look at the conversion registers 
    thread::sleep(Duration::from_millis(DELAY_TIME));

    i2c0.block_read(REG_CONVERSION, &mut adc2_2_reg)?;       // reads ADS115 conversion register and puts contents into reg buffer
    thread::sleep(Duration::from_millis(DELAY_TIME));


    let mut adc2_2_val:u16 = u16::from_be_bytes(adc2_2_reg);
    //println!(" ADC 0 decimal value = {:?} ", adc0val);
    let mut adc2_2_voltage:f32 = adc2_2_val.into(); 

    let mut adc2_2_voltage:f32 = adc2_2_voltage * 0.000125;
    println!(" ADC_2 2 voltage = {:?} ", adc2_2_voltage);
    
    Ok(())

}


fn get_adc3_2_value() -> Result<(), Box<dyn Error>>  // this is a second ADS1115 ADC slave chip
{

    let mut adc3_2_reg = [0u8; 2];

    let mut i2c0 = I2c::new()?;
    i2c0.set_slave_address(ADDR_ADS115_TWO)?;

    i2c0.block_write(REG_CONFIGURATION, &[0x72, 0x82],)?; // Set configuration setting to ADS115
    thread::sleep(Duration::from_millis(DELAY_TIME));

    i2c0.block_write(REG_CONVERSION, &[0x00],)?;       // Set ADS115 config to look at the conversion registers 
    thread::sleep(Duration::from_millis(DELAY_TIME));

    i2c0.block_read(REG_CONVERSION, &mut adc3_2_reg)?;       // reads ADS115 conversion register and puts contents into reg buffer
    thread::sleep(Duration::from_millis(DELAY_TIME));


    let mut adc3_2_val:u16 = u16::from_be_bytes(adc3_2_reg);
    //println!(" ADC 0 decimal value = {:?} ", adc0val);
    let mut adc3_2_voltage:f32 = adc3_2_val.into(); 

    let mut adc3_2_voltage:f32 = adc3_2_voltage * 0.000125;
    println!(" ADC_2 3 voltage = {:?} ", adc3_2_voltage);
    
    Ok(())

}