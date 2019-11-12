from ffi import ffi
import _numpypy as np

def entrypoint(argv):
    context_id, context_token = argv
    x = get_matrix(10)
    y = get_matrix(10)
    z = x.dot(y)
    return str(z)

def get_matrix(n):
    x = np.multiarray.zeros((n, n))
    for i in range(n):
        for j in range(n):
            x[i][j] = i * j
    return x
