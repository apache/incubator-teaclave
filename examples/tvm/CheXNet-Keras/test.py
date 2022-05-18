import numpy as np
import os
from configparser import ConfigParser
from generator import AugmentedImageSequence
from models.kr import ModelFactory
from sklearn.metrics import roc_auc_score
from utility import get_sample_counts
import sys
import sklearn

# apply threshold to positive probabilities to create labels
def to_labels(pos_probs, threshold):
	return (pos_probs >= threshold).astype('int')

def main():
    # parser config
    config_file = "./config.ini"
    cp = ConfigParser()
    cp.read(config_file)

    # default config
    output_dir = cp["DEFAULT"].get("output_dir")
    base_model_name = cp["DEFAULT"].get("base_model_name")
    class_names = cp["DEFAULT"].get("class_names").split(",")
    image_source_dir = cp["DEFAULT"].get("image_source_dir")

    # train config
    image_dimension = cp["TRAIN"].getint("image_dimension")

    # test config
    batch_size = cp["TEST"].getint("batch_size")
    test_steps = cp["TEST"].get("test_steps")
    use_best_weights = cp["TEST"].getboolean("use_best_weights")

    # parse weights file path
    output_weights_name = cp["TRAIN"].get("output_weights_name")
    weights_path = os.path.join(output_dir, output_weights_name)
    best_weights_path = os.path.join(output_dir, f"best_{output_weights_name}")

    # get test sample count
    test_counts, _ = get_sample_counts(output_dir, "test", class_names)

    # compute steps
    if test_steps == "auto":
        test_steps = int(test_counts / batch_size)
    else:
        try:
            test_steps = int(test_steps)
        except ValueError:
            raise ValueError(f"""
                test_steps: {test_steps} is invalid,
                please use 'auto' or integer.
                """)
    print(f"** test_steps: {test_steps} **")

    print("** load model **")
    if use_best_weights:
        print("** use best weights **")
        model_weights_path = best_weights_path
    else:
        print("** use last weights **")
        model_weights_path = weights_path
    model_factory = ModelFactory()
    model = model_factory.get_model(
        class_names,
        model_name=base_model_name,
        use_base_weights=False,
        weights_path=model_weights_path)

    print("** load test generator **")
    test_sequence = AugmentedImageSequence(
        dataset_csv_file=os.path.join(output_dir, "dev.csv"),
        class_names=class_names,
        source_image_dir=image_source_dir,
        batch_size=batch_size,
        target_size=(image_dimension, image_dimension),
        augmenter=None,
        steps=test_steps,
        shuffle_on_epoch_end=False,
    )

    np.set_printoptions(threshold=sys.maxsize)
    print("** make prediction **")
    y_hat = model.predict(test_sequence, verbose=1)
    print(f"ndim: {y_hat.ndim}, shape: {y_hat.shape}")
    y = test_sequence.get_y_true()
    with open("/tmp/test.log", "w") as f:
        for i in y_hat:
            f.write(f"{i}\n")
    with open("/tmp/test_max_idx.log", "w") as f:
        for i in y_hat:
            idx = np.argmax(i)
            f.write(f"{idx}\n")


    test_log_path = os.path.join(output_dir, "test.log")
    print(f"** write log to {test_log_path} **")
    aurocs = []
    with open(test_log_path, "w") as f:
        for i in range(len(class_names)):
            try:
                score = roc_auc_score(y[:, i], y_hat[:, i])
                # approach 1: https://machinelearningmastery.com/threshold-moving-for-imbalanced-classification/
                #fpr, tpr, thresholds = sklearn.metrics.roc_curve(y[:, i], y_hat[:, i])
                #f.write(f"{class_names[i]}: {fpr} \n")
                #f.write(f"{class_names[i]}: {tpr} \n")
                #f.write(f"{class_names[i]}: {thresholds} \n")
                #gmeans = np.sqrt(tpr * (1-fpr))
                #ix = np.argmax(gmeans)
                #f.write("Best Threshold=%f, G-Mean=%.3f\n" % (thresholds[ix], gmeans[ix]))

                # approach 2: f1_score 
                # keep probabilities for the positive outcome only
                probs = y_hat[:, i]
                testy = y[:, i]
                # define thresholds
                thresholds = np.arange(0, 1, 0.001)
                # evaluate each threshold
                evl_scores = [sklearn.metrics.f1_score(testy, to_labels(probs, t)) for t in thresholds]
                # get best threshold
                ix = np.argmax(evl_scores)
                print('Threshold=%.3f, F-Score=%.5f. Class: %s' % (thresholds[ix], evl_scores[ix], class_names[i]))

                aurocs.append(score)
            except ValueError:
                score = -1
            f.write(f"{class_names[i]}: {score}\n")
        mean_auroc = np.mean(aurocs)
        f.write("-------------------------\n")
        f.write(f"mean auroc: {mean_auroc}\n")
        print(f"mean auroc: {mean_auroc}")


if __name__ == "__main__":
    main()
