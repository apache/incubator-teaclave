import mesatee
from ffi import ffi
import _numpypy as np
import marshal


def read_file_train(context_id, context_token, file_id):
    content = mesatee.mesatee_read_file(context_id, context_token, file_id)
    featureData=[]
    labelData=[]
    lines = content.strip().split('\n')
    for line in lines:
        line = line.strip().split(',')
        featureData.append(line[:-1])
        labelData.append(line[-1])
    label = np.multiarray.array(labelData, dtype='float64').reshape(-1,1)
    feature = np.multiarray.array(featureData, dtype='float64')
    return feature, label

def read_file_predict(context_id, context_token, file_id, params_id, scaler_id):
    params = mesatee.mesatee_read_file(context_id, context_token, params_id) 
    scaler = mesatee.mesatee_read_file(context_id, context_token, scaler_id)
    params = marshal.loads(params)
    scaler = marshal.loads(scaler)
    content = mesatee.mesatee_read_file(context_id, context_token, file_id)
    featureData=[]
    labelData=[]
    lines = content.strip().split('\n')
    for line in lines:
        line = line.strip().split(',')
        featureData.append(line)

    feature = np.multiarray.array(featureData, dtype='float64')
    return feature, params, scaler

def save_model(context_id, context_token, params, scaler):
    params = marshal.dumps(params)
    scaler = marshal.dumps(scaler)
    params_saved_id = mesatee.mesatee_save_file_for_task_creator(context_id, context_token, params)
    scaler_saved_id = mesatee.mesatee_save_file_for_task_creator(context_id, context_token, scaler)
    return params_saved_id, scaler_saved_id


def minmaxscaler_train(input_array):
    array_max = input_array.max(0)
    array_min = input_array.min(0)
    scaler = {
        "max": array_max,
        "min": array_min
    }
    input_array = (input_array - array_min) / (array_max - array_min)
    return input_array, scaler

def minmaxscaler_predict(input_array, scaler):
    array_max = np.multiarray.frombuffer(scaler["max"])
    array_min = np.multiarray.frombuffer(scaler["min"])
    input_array = (input_array - array_min) / (array_max - array_min)
    return input_array 


def sigmoid(z):
    a = 1/(1+np.umath.exp(-z))
    return a

def initialize_with_zeros(dim):
    w = np.multiarray.zeros((dim,1))
    b = 0
    return w,b

def propagate(w, b, X, Y):
    m = X.shape[1]
    A = sigmoid(w.T.dot(X) + b)  
    cost = -((Y * np.umath.log(A) + (1-Y) * np.umath.log(1-A)).sum())/m  

    dZ = A-Y  
    dw = (X.dot(dZ.T))/m
    db = (dZ.sum())/m

    grads = {"dw": dw,
             "db": db}
    return grads, cost

def optimize(w, b, X, Y, num_iterations, learning_rate):
    costs = []
    for i in range(num_iterations):
        grads, cost = propagate(w,b,X,Y)
        dw = grads["dw"]
        db = grads["db"]
        w = w - learning_rate*dw
        b = b - learning_rate*db
    params = {"w": w,
              "b": b}
    return params


def logistic_model(feature, label, learning_rate=0.1, num_iterations=2000):
    dim = feature.shape[0]
    w,b = initialize_with_zeros(dim)

    params = optimize(w,b,feature,label,num_iterations,learning_rate)

    return params

def logistic_predict(feature, params):
    w = np.multiarray.frombuffer(params['w'])
    b = np.multiarray.frombuffer(params['b'])
    m = feature.shape[1]
    prediction = np.multiarray.zeros((1,m))

    A = sigmoid(w.T.dot(feature) + b) 
    for i in range(m):
        if A[i]>0.5:
            prediction[0,i] = 1
        else:
            prediction[0,i] = 0

    return prediction


def train(context_id, context_token, file_id):
    feature, label = read_file_train(context_id, context_token, file_id)
    feature, scaler = minmaxscaler_train(feature)
    feature = feature.T
    label = label.T
    params = logistic_model(feature, label, num_iterations = 2000, learning_rate = 0.05)
    params_saved_id, scaler_saved_id = save_model(context_id, context_token, params, scaler)
    return params_saved_id, scaler_saved_id
    
def predict(context_id, context_token, file_id, params_id, scaler_id):
    feature, params, scaler = read_file_predict(context_id, context_token, file_id, params_id, scaler_id)
    feature = minmaxscaler_predict(feature, scaler)
    feature = feature.T
    prediction = logistic_predict(feature, params)
    return prediction


def entrypoint(argv):
    context_id, context_token, train_file_id, predict_file_id = argv
    params_saved_id, scaler_saved_id = train(context_id, context_token, train_file_id)
    prediction = predict(context_id, context_token, predict_file_id, params_saved_id, scaler_saved_id)
    return str(prediction)
