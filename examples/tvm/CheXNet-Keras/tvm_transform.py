from models.kr import ModelFactory

import tvm
import tvm.relay as relay
import os
import subprocess


def run():
    cls_name_str = "Atelectasis,Cardiomegaly,Effusion,Infiltration,Mass,Nodule,Pneumonia,Pneumothorax,Consolidation,Edema,Emphysema,Fibrosis,Pleural_Thickening,Hernia"
    class_names = cls_name_str.split(',')
    base_model_name = "DenseNet121"
    model_weights_path = "best_weights.h5"

    model_factory = ModelFactory()
    model = model_factory.get_model(
        class_names,
        model_name=base_model_name,
        use_base_weights=False,
        weights_path=model_weights_path)
    

    input_shape=(1, 3, 224, 224)
    shape_dict = {"input_1": input_shape}
    mod, params = relay.frontend.from_keras(model, shape_dict)

    target = "llvm --system-lib"
    with tvm.transform.PassContext(opt_level=0):
        lib = relay.build(mod, target=target, params=params)


    out_dir = "."

    obj_file = os.path.join(out_dir, "graph_native.o")

    lib.get_lib().save(obj_file)
    graph_json = lib.get_graph_json()
    params = lib.get_params()

    # Run llvm-ar to archive obj_file into lib_file
    lib_file = os.path.join(out_dir, "libgraph_native.a")
    cmds = [os.environ.get("LLVM_AR", "llvm-ar-10"), "rcs", lib_file, obj_file]
    subprocess.run(cmds)

    with open(os.path.join(out_dir, "graph.json"), "w") as f_graph:
        f_graph.write(graph_json)

    with open(os.path.join(out_dir, "graph.params"), "wb") as f_params:
        f_params.write(tvm.runtime.save_param_dict(params))

run()
