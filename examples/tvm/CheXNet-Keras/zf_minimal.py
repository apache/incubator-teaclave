
import pandas as pd
from tensorflow.keras.layers import Input
from tensorflow.keras.layers import Dense
from tensorflow.keras.models import Model
from tensorflow.keras.applications import DenseNet121

CLASS_NAMES = ['Atelectasis', 'Cardiomegaly', 'Effusion', 
	'Infiltration', 'Mass', 'Nodule', 'Pneumonia', 
	'Pneumothorax', 'Consolidation', 'Edema', 
	'Emphysema', 'Fibrosis', 'Pleural_Thickening', 'Hernia']

IMAGE_DIR = "data/images"
WEIGHT_PATH = "best_weights.h5"

INPUT_SHAPE = (224, 224, 3)

#https://machinelearningmastery.com/threshold-moving-for-imbalanced-classification/
#In test.py
#gmeans = np.sqrt(tpr * (1-fpr))
#ix = np.argmax(gmeans)
#f.write("Best Threshold=%f, G-Mean=%.3f\n" % (thresholds[ix], gmeans[ix]))
THRESHOLDS = [
 0.107043, 0.013231, 0.125283, 0.169727, 0.057182, 0.046832, 0.015987,
 0.062943, 0.028716, 0.039941, 0.029412, 0.018853, 0.040653, 0.005409
]

#Threshold=0.163, F-Score=0.39925. Class: Atelectasis
#Threshold=0.257, F-Score=0.28571. Class: Cardiomegaly
#Threshold=0.217, F-Score=0.54209. Class: Effusion
#Threshold=0.170, F-Score=0.43059. Class: Infiltration
#Threshold=0.212, F-Score=0.41667. Class: Mass
#Threshold=0.123, F-Score=0.31915. Class: Nodule
#Threshold=0.031, F-Score=0.11173. Class: Pneumonia
#Threshold=0.155, F-Score=0.45033. Class: Pneumothorax
#Threshold=0.156, F-Score=0.16250. Class: Consolidation
#Threshold=0.112, F-Score=0.30476. Class: Edema
#Threshold=0.231, F-Score=0.54167. Class: Emphysema
#Threshold=0.098, F-Score=0.17647. Class: Fibrosis
#Threshold=0.120, F-Score=0.25688. Class: Pleural_Thickening
#Threshold=0.057, F-Score=0.44444. Class: Hernia

THRESHOLDS = [0.163, 0.257, 0.217, 0.170, 0.212, 0.123, 0.031,
0.155, 0.156, 0.112, 0.231, 0.098, 0.120, 0.057]

IMAGES = [
# Atelectasis
"00013118_008.png",
"00014716_007.png",
"00029817_009.png",
"00014687_001.png",
"00017877_001.png",
"00003148_004.png",
"00012515_002.png",
"00022098_006.png",
"00014198_000.png",
"00021007_000.png",

# Cardiomegaly
"00016990_000.png",
"00027797_000.png",
"00013670_151.png",
"00011322_006.png",
"00018387_030.png",
"00007037_000.png",
"00017448_000.png",
"00029808_003.png",
"00013249_031.png",

# Effusion
"00015078_013.png",
"00020000_000.png",
"00010277_000.png", 
"00030634_000.png",
"00018427_011.png",
"00016837_002.png",
"00013285_026.png",
"00019767_016.png",
"00018366_029.png",
"00003803_010.png",
"00002395_007.png",

#Infiltrate
"00026911_000.png",
"00029404_004.png",
"00019363_043.png",
"00006096_010.png",
"00003333_002.png",
"00011269_018.png",
"00016786_009.png",
"00013106_000.png",
"00019706_012.png",
"00015831_011.png",
"00020482_011.png",
"00016732_040.png",
"00021926_007.png",

#Mass
"00022192_003.png",
"00022237_002.png",
"00022726_002.png",
"00000830_000.png",
"00015440_000.png",
"00011925_076.png",
"00017611_002.png",
"00029469_007.png",
"00014551_010.png",
"00027697_001.png",

#Nodule
"00017243_010.png",
"00025662_006.png",
"00027470_006.png",
"00019013_002.png",
"00015141_002.png",
"00026398_000.png",
"00021374_000.png",
"00017199_005.png",
"00015583_000.png",
"00017098_003.png",
"00019177_000.png",
"00030162_029.png",
"00026285_000.png",
"00002578_000.png",
"00013951_001.png",
"00020393_001.png",
"00015018_004.png",

#Pneumonia
"00019157_008.png",
"00009229_003.png",
"00007444_003.png",
"00001933_000.png",
"00002711_000.png",
"00009107_006.png",
"00027758_004.png",
"00013249_033.png",
"00023089_004.png",
"00021860_003.png",
"00013993_013.png",
"00010447_018.png",
"00008727_009.png",
]



def load_model(weights_path):
    # load model
    img_input = Input(shape=INPUT_SHAPE)

    base_model = DenseNet121(
            include_top=False,
            input_tensor=img_input,
            input_shape=INPUT_SHAPE,
            weights=None,
            pooling="avg")
    x = base_model.output
    predictions = Dense(len(CLASS_NAMES), activation="sigmoid", name="predictions")(x)
    #predictions = Dense(len(class_names), activation="sigmoid", name="predictions")(base_model.layers[-1].output)
    model = Model(inputs=img_input, outputs=predictions)
    model.load_weights(weights_path)
    return model


# load image
from PIL import Image
import numpy as np
from skimage.transform import resize
import os

def load_image(image_path):
    target_size= INPUT_SHAPE[:2]
    image = Image.open(image_path)
    image_array = np.asarray(image.convert("RGB"))
    image_array = image_array / 255.
    image_array = resize(image_array, target_size)
    image_array = transform_image(image_array)
    return image_array

def transform_image(image_array):
    imagenet_mean = np.array([0.485, 0.456, 0.406])
    imagenet_std = np.array([0.229, 0.224, 0.225])
    return (image_array - imagenet_mean) / imagenet_std

def get_image_index(df_images, file_name):
    for (k, v) in df_images["file_name"].to_dict().items():
        if v == file_name:
            return k
    return -1 
 
 

# labeled bbox
def ground_truth_for_image(image_path):
    image_name = os.path.basename(image_path)
    bbox_list_file = "data/BBox_List_2017.csv"
    df_images = pd.read_csv(bbox_list_file, header=None, skiprows=1)
    df_images.columns = ["file_name", "label", "x", "y", "w", "h"]
    idx = get_image_index(df_images, image_name)
    if idx == -1:
        return -1, "NF", None
    label_info = df_images.iloc[idx]

    expected_label = label_info["label"]
    if expected_label == "Infiltrate":
        expected_label = "Infiltration"

    # bbox info
    # x,y,w,h = label_info["x"], label_info["y"], label_info["w"], label_info["h"]

    expected_index = CLASS_NAMES.index(expected_label)
    return expected_index, expected_label, label_info

def get_output_layer(model, layer_name):
    # get the symbolic outputs of each "key" layer (we gave them unique names).
    layer_dict = dict([(layer.name, layer) for layer in model.layers])
    layer = layer_dict[layer_name]
    return layer


from tensorflow.keras import backend as kb
import cv2
def do_cam(model, image_path, predicted_index, expected_label, label_info, output_dir = "/tmp"):
    # CAM overlay
    # Get the 512 input weights to the softmax.
    img = load_image(image_path)
    class_weights = model.layers[-1].get_weights()[0]
    final_conv_layer = get_output_layer(model, "bn")
    get_output = kb.function([model.layers[0].input], [final_conv_layer.output, model.layers[-1].output]) 
    [conv_outputs, predictions] = get_output([np.array([img])]) 
    conv_outputs = conv_outputs[0, :, :, :]

    # output
    image_name = os.path.basename(image_path)
    output_path = os.path.join(output_dir, f"{expected_label}.{image_name}")

    img_ori = cv2.imread(filename=image_path)

    index = predicted_index

    # Create the class activation map.
    cam = np.zeros(dtype=np.float32, shape=(conv_outputs.shape[:2]))
    for i, w in enumerate(class_weights[:,index]):
        cam += w * conv_outputs[:, :, i]
    cam /= np.max(cam)
    cam = cv2.resize(cam, img_ori.shape[:2])
    heatmap = cv2.applyColorMap(np.uint8(255 * cam), cv2.COLORMAP_JET)
    heatmap[np.where(cam < 0.2)] = 0
    img = heatmap * 0.5 + img_ori

    predicted_label = CLASS_NAMES[predicted_index]

    # add label & rectangle
    # ratio = output dimension / 1024
    ratio = 1
    x1 = int(label_info["x"] * ratio)
    y1 = int(label_info["y"] * ratio)
    x2 = int((label_info["x"] + label_info["w"]) * ratio)
    y2 = int((label_info["y"] + label_info["h"]) * ratio)
    cv2.rectangle(img, (x1, y1), (x2, y2), (255, 0, 0), 2)
    cv2.putText(img, text=predicted_label, org=(5, 20), fontFace=cv2.FONT_HERSHEY_SIMPLEX,
                fontScale=0.8, color=(0, 0, 255), thickness=1)
    cv2.imwrite(output_path, img)
    print(f"Cam and BBox added to {output_path}")


import argparse
def arg_parse():
    parser = argparse.ArgumentParser(description='CheXNet Prediction')
    parser.add_argument('--image', help='image path')
    parser.add_argument('--weight',  default= WEIGHT_PATH,
                        help = 'path for model weights')
    parser.add_argument('--builtin', nargs = "?", const = True, default = None, help='')
    parser.add_argument('--output_dir', help = 'Directory for output image')
    return parser.parse_args()

def main():
    args = arg_parse()
    
    if args.builtin:
        model = load_model(WEIGHT_PATH)
        for img_name in IMAGES:
            image_path = os.path.join(IMAGE_DIR, img_name)
            image_array = load_image(image_path)
            pd_result = model.predict([np.array([image_array])])
            result = pd_result[0]
            #print(f"result: {result}")
            pd_idxes = [1 if result[i] > THRESHOLDS[i] else 0 for i in range(len(result))]
            predicted_index = [i for i,x in enumerate(pd_idxes) if x==1] 
            #predicted_index = np.argmax(pd_result)
            expected_index, expected_label, label_info = ground_truth_for_image(image_path)
            print(f"Predicted == {predicted_index} vs {expected_index} == Expected. Image: {image_path}")

    else: 
        image_path = args.image
        weight_path = args.weight
        model = load_model(weight_path)
        image_array = load_image(image_path)

        pd_result = model.predict([np.array([image_array])])
        print(f"pd_result: {pd_result}")
        result = pd_result[0]
        pd_idxes = [1 if result[i] > THRESHOLDS[i] else 0 for i in range(len(result))]
        predicted_index = [i for i,x in enumerate(pd_idxes) if x==1] 

        expected_index, expected_label, label_info = ground_truth_for_image(image_path)
        print(f"Predicted == {predicted_index}; Expected == {expected_index}")
        if label_info is not None:
            do_cam(model, image_path, expected_index, expected_label, label_info)


if __name__ == "__main__":
    main()