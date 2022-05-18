
use image::{FilterType, GenericImageView};

extern {
    fn do_inference(eid: sgx_enclave_id_t, retval: *mut sgx_status_t,
                     some_string: *const u8, len: usize) -> sgx_status_t;
}

fn preprocess(img: image::DynamicImage) -> Vec<f32> {
    println!("original image dimensions: {:?}", img.dimensions());
    let img = img
        .resize_exact(IMG_HEIGHT as u32, IMG_WIDTH as u32, FilterType::Nearest)
        .to_rgb();
    println!("resized image dimensions: {:?}", img.dimensions());
    let mut pixels: Vec<f32> = vec![];
    for pixel in img.pixels() {
        let tmp = pixel.data;
        // normalize the RGB channels using mean, std of imagenet1k
        let tmp = [
            (tmp[0] as f32 - 123.0) / 58.395, // R
            (tmp[1] as f32 - 117.0) / 57.12,  // G
            (tmp[2] as f32 - 104.0) / 57.375, // B
        ];
        for e in &tmp {
            pixels.push(*e);
        }
    }

    println!("[+] pixels: {:?}", pixels.len());
    pixels
}

fn do_ecall() {
    let img_bytes = include_bytes!("../test_data/cat.png");
    let img = image::load_from_memory(img_bytes.as_ref()).unwrap();
    let input = preprocess(img);
    //ecall
}