
use sgx_types::*;
use image::{FilterType, GenericImageView};

const IMG_HEIGHT: usize = 224;
const IMG_WIDTH: usize = 224;

extern {
    fn do_inference(eid: sgx_enclave_id_t, retval: *mut i32,
                     input: *const f32, len: usize) -> sgx_status_t;
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

pub fn sgx_inference(eid: sgx_enclave_id_t) {
    let img_bytes = include_bytes!("../cat.png");
    let img = image::load_from_memory(img_bytes.as_ref()).unwrap();
    let input = preprocess(img);

    let mut ret: i32 = -1;

    let result = unsafe {
        do_inference(eid, &mut ret, input.as_ptr() as * const f32, input.len())
    };

    match result {
        sgx_status_t::SGX_SUCCESS => {
            println!("[+] ECALL Enclave Succeed {}!", ret);
        },
        _ => {
            println!("[-] ECALL Enclave Failed {}!", result.as_str());
            return;
        }
    }
    return;
}