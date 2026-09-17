#include "HEaaN/Context.hpp"
#include "HEaaN/KeyGenerator.hpp"
#include "HEaaN/KeyPack.hpp"
#include "HEaaN/MatrixCiphertext.hpp"
#include "HEaaN/ParameterPreset.hpp"
#include "HEaaN/SecretKey.hpp"

#include "LargeMatrixEnDecryptor.hpp"
#include "MatrixEvaluator.hpp"
#include "RealMatrix.hpp"

#include <algorithm>
#include <cmath>
#include <iostream>

namespace {

constexpr HEaaN::u64 INIT_LEVEL = 1;

double maxAbsError(
    const HEaaN::RealMatrix &expected,
    const HEaaN::RealMatrix &actual)
{
    double error = 0.0;

    for (HEaaN::u64 col = 0; col < expected.getNumColumn(); ++col) {
        for (HEaaN::u64 row = 0; row < expected.getNumRow(); ++row) {
            error = std::max(
                error,
                std::abs(
                    static_cast<double>(expected.getEntry(row, col))
                    - static_cast<double>(actual.getEntry(row, col))));
        }
    }

    return error;
}

} // namespace

int main()
{
    const auto preset = HEaaN::ParameterPreset::SN1;
    const auto context = HEaaN::makeContext(preset);
    const HEaaN::u64 dim = HEaaN::getDegree(context);

    std::cout << "CCMM_ORACLE_DIM=" << dim << '\n';

    HEaaN::KeyPack pack(context);
    HEaaN::SecretKey sk(context);

    HEaaN::KeyGenerator keygen(context, sk, pack);
    keygen.genEncKey();
    keygen.genMultKey();

    std::cout << "CCMM_ORACLE_KEYGEN_BEGIN\n";

    for (HEaaN::u64 aut = 1; aut < 2 * dim; aut += 2) {
        keygen.genAutKey(aut);
    }

    std::cout << "CCMM_ORACLE_KEYGEN_END\n";

    HEaaN::LargeMatrixEnDecryptor encdec(context);
    HEaaN::MatrixEvaluator evaluator(context, pack);

    HEaaN::RealMatrix lhs(dim, dim);
    HEaaN::RealMatrix rhs(dim, dim);
    HEaaN::RealMatrix expected(dim, dim);
    HEaaN::RealMatrix actual(dim, dim);

    for (HEaaN::u64 col = 0; col < dim; ++col) {
        for (HEaaN::u64 row = 0; row < dim; ++row) {
            lhs.getEntry(row, col) = 0.0;
            rhs.getEntry(row, col) = 0.0;
            expected.getEntry(row, col) = 0.0;
            actual.getEntry(row, col) = 0.0;
        }
    }

    // A = [[1, 2],
    //      [3, 4]]
    lhs.getEntry(0, 0) = 1.0;
    lhs.getEntry(1, 0) = 3.0;
    lhs.getEntry(0, 1) = 2.0;
    lhs.getEntry(1, 1) = 4.0;

    // B = [[5, 6],
    //      [7, 8]]
    rhs.getEntry(0, 0) = 5.0;
    rhs.getEntry(1, 0) = 7.0;
    rhs.getEntry(0, 1) = 6.0;
    rhs.getEntry(1, 1) = 8.0;

    HEaaN::naiveMatrixMult(lhs, rhs, expected);

    HEaaN::MatrixCiphertext ctLhs(context, dim);
    HEaaN::MatrixCiphertext ctRhs(context, dim);
    HEaaN::MatrixCiphertext ctOut(context, dim);

    std::cout << "CCMM_ORACLE_ENCRYPT_BEGIN\n";

    encdec.encrypt(lhs, sk, ctLhs, INIT_LEVEL);
    encdec.encrypt(rhs, sk, ctRhs, INIT_LEVEL);

    std::cout << "CCMM_ORACLE_ENCRYPT_END\n";
    std::cout << "CCMM_ORACLE_COMPUTE_BEGIN\n";

    evaluator.matrixMult(ctLhs, ctRhs, ctOut);

    std::cout << "CCMM_ORACLE_COMPUTE_END\n";
    std::cout << "CCMM_ORACLE_DECRYPT_BEGIN\n";

    encdec.decrypt(ctOut, sk, actual);

    std::cout << "CCMM_ORACLE_DECRYPT_END\n";

    const double error = maxAbsError(expected, actual);

    std::cout << "EXPECTED_00=" << expected.getEntry(0, 0) << '\n';
    std::cout << "EXPECTED_01=" << expected.getEntry(0, 1) << '\n';
    std::cout << "EXPECTED_10=" << expected.getEntry(1, 0) << '\n';
    std::cout << "EXPECTED_11=" << expected.getEntry(1, 1) << '\n';

    std::cout << "ACTUAL_00=" << actual.getEntry(0, 0) << '\n';
    std::cout << "ACTUAL_01=" << actual.getEntry(0, 1) << '\n';
    std::cout << "ACTUAL_10=" << actual.getEntry(1, 0) << '\n';
    std::cout << "ACTUAL_11=" << actual.getEntry(1, 1) << '\n';

    std::cout << "MAX_ABS_ERROR=" << error << '\n';

    const double tolerance = 1e-3;
    const bool pass = error < tolerance;

    std::cout << "CCMM_ORACLE=" << (pass ? "PASS" : "FAIL") << '\n';

    return pass ? 0 : 1;
}
