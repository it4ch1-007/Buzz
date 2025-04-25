use cpal::traits::{DeviceTrait,HostTrait,StreamTrait};
use hound::{WavWriter,WavSpec};


pub fn upload_audio_msg() -> Result<(String,Vec<u8>),std::io::Error>{
    //STORE IT ON THE SERVER FIRST IN THE FORMAT OF WAV OR ANOTHER SOUND FILE FORMAT

    //Initializing the host
    let default_host = cpal::default_host();
    let output_device = default_host.default_output_device().expect("No output device detected!!");
    let input_device = default_host.default_input_device().expect("No input device detected!!");
    let input_config = input_device.default_input_config().unwrap();
    
    Ok(("test".to_string(),vec![0u8,1u8]))
}
pub fn play_audio_msg(audio_msg: String){

}

