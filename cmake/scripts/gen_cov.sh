#!/bin/bash
set -e
REQUIRED_ENVS=("MESATEE_PROJECT_ROOT" "MESATEE_OUT_DIR" "MESATEE_TARGET_DIR")
for var in "${REQUIRED_ENVS[@]}"; do
    [ -z "${!var}" ] && echo "Please set ${var}" && exit -1
done



LCOV=lcov
LCOVOPT="--gcov-tool ${MESATEE_PROJECT_ROOT}/build/llvm-gcov"
GENHTML=genhtml

cd ${MESATEE_PROJECT_ROOT}
find . \( -name "*.gcda" -and \( ! -name "sgx_cov*" \
    -and ! -name "kms*" -and ! -name "fns*" \
    -and ! -name "tdfs*" -and ! -name "tms*" \
    -and ! -name "private_join_and_compute*"\
    -and ! -name "ml_predict*"\
    -and ! -name "online_decrypt*"\
    -and ! -name "image_resizing*"\
    -and ! -name "kmeans*"\
    -and ! -name "logistic_reg*"\
    -and ! -name "lin_reg*"\
    -and ! -name "svm*"\
    -and ! -name "gen_linear_model*"\
    -and ! -name "gaussian_mixture_model*"\
    -and ! -name "gaussian_processes*"\
    -and ! -name "dbscan*"\
    -and ! -name "neural_net*"\
    -and ! -name "naive_bayes*"\
    -and ! -name "mesatee_core*" -and ! -name "mesatee_config*" \) \) \
    -exec rm {} \;
cd ${MESATEE_PROJECT_ROOT} && \
    for tag in `find ${MESATEE_PROJECT_ROOT} -name sgx_cov*.gcda | cut -d'.' -f2`; \
    do mkdir -p ${MESATEE_OUT_DIR}/cov_$tag && \
    find ${MESATEE_TARGET_DIR} -name *$tag* -exec mv {} ${MESATEE_OUT_DIR}/cov_$tag/ \; ; \
    ${LCOV} ${LCOVOPT} --capture \
    --directory ${MESATEE_OUT_DIR}/cov_$tag/ --base-directory . \
    -o ${MESATEE_OUT_DIR}/modules_$tag.info; done 2>/dev/null
rm -rf ${MESATEE_OUT_DIR}/cov_*
cd ${MESATEE_PROJECT_ROOT} && ${LCOV} ${LCOVOPT} --capture \
    --directory . --base-directory . \
    -o ${MESATEE_OUT_DIR}/modules.info 2>/dev/null
cd ${MESATEE_OUT_DIR} && ${LCOV} ${LCOVOPT} $(for tag in \
    `find ${MESATEE_PROJECT_ROOT} -name sgx_cov*.gcda | cut -d'.' -f2`; \
    do echo "--add modules_$tag.info"; done) \
    --add modules.info -o merged.info
${LCOV} ${LCOVOPT} --extract ${MESATEE_OUT_DIR}/merged.info \
    `find ${MESATEE_PROJECT_ROOT} -path ${MESATEE_PROJECT_ROOT}/third_party -prune -o \
    -name "*.rs"` -o cov.info
${GENHTML} --branch-coverage --demangle-cpp --legend cov.info \
    -o cov_report --ignore-errors source
