# Todo API - Production-Ready Testing Infrastructure

Bu proje, Rust Axum framework'ü kullanılarak geliştirilmiş bir Todo API'sının kapsamlı test altyapısını içerir. Production-ready ve best practice'lere uygun olarak tasarlanmıştır.

## 🚀 Test Altyapısı Özellikleri

### 📊 Test Kategorileri

1. **Unit Tests (Birim Testleri)**
   - JWT Service testleri
   - Auth Service testleri  
   - Todo Model testleri
   - Business logic testleri

2. **Integration Tests (Entegrasyon Testleri)**
   - Service katmanları arası etkileşim testleri
   - Model dönüşüm testleri
   - Authentication flow testleri
   - Complete lifecycle testleri

3. **Performance Tests (Performans Testleri)**
   - JWT token generation benchmarks
   - JWT token verification benchmarks
   - Model creation benchmarks
   - Request processing benchmarks

### 🛠 Kullanılan Test Framework'leri

- **tokio-test**: Async testing framework
- **serial_test**: Database isolation için seri test execution
- **criterion**: Benchmarking ve performance testing
- **fake**: Realistic test data generation
- **proptest**: Property-based testing
- **mockall**: Service mocking
- **testcontainers**: Isolated database testing (opsiyonel)

## 📁 Test Dosya Yapısı

```
tests/
├── unit_jwt_service.rs      # JWT servis unit testleri
├── unit_auth_service.rs     # Auth servis unit testleri
├── unit_todo_models.rs      # Todo model unit testleri
├── integration_basic.rs     # Temel integration testleri
└── common/                  # Test utilities ve helpers
    ├── mod.rs
    ├── database.rs
    ├── fixtures.rs
    ├── auth_helpers.rs
    └── test_client.rs

benches/
└── performance_tests.rs     # Performance benchmarks

scripts/
└── test_runner.sh          # Test automation script
```

## 🚀 Test Çalıştırma

### Manual Test Commands

```bash
# Tüm testleri çalıştır
cargo test

# Sadece unit testleri
cargo test unit_

# Sadece integration testleri  
cargo test integration_

# Performance benchmarks
cargo bench

# Specific test dosyası
cargo test --test unit_jwt_service
```

### Test Runner Script

Comprehensive test runner script ile kolay test yönetimi:

```bash
# Script'e executable izin ver
chmod +x scripts/test_runner.sh

# Tüm testleri çalıştır
./scripts/test_runner.sh all

# Sadece unit testler
./scripts/test_runner.sh unit

# Sadece integration testler
./scripts/test_runner.sh integration

# Performance benchmarks
./scripts/test_runner.sh bench

# Test coverage raporu
./scripts/test_runner.sh coverage

# Code linting (clippy + fmt)
./scripts/test_runner.sh lint

# Security audit
./scripts/test_runner.sh audit

# Test istatistikleri
./scripts/test_runner.sh stats

# Test artifacts temizle
./scripts/test_runner.sh clean

# Test watcher (otomatik test çalıştırma)
./scripts/test_runner.sh watch

# Full CI pipeline
./scripts/test_runner.sh ci

# Yardım
./scripts/test_runner.sh help
```

## 📊 Test Coverage

Test coverage raporu oluşturmak için:

```bash
# cargo-llvm-cov install (bir kere)
cargo install cargo-llvm-cov

# Coverage raporu oluştur
cargo llvm-cov --html --output-dir coverage/

# Coverage raporunu aç
open coverage/index.html
```

## 🧪 Test Verileri

Test verileri için kullanılan yaklaşımlar:

1. **Fixed Test Data**: Tahmin edilebilir sonuçlar için sabit test verileri
2. **Generated Test Data**: `fake` crate ile realistic test data
3. **Property-based Tests**: `proptest` ile edge case testing
4. **Mock Services**: `mockall` ile external dependency mocking

## 🏗 Test Architecture

### Unit Tests
- **Scope**: Tek bir component/function
- **Dependencies**: Mock edilmiş
- **Speed**: Çok hızlı
- **Purpose**: Business logic validation

### Integration Tests  
- **Scope**: Birden fazla component arası etkileşim
- **Dependencies**: Real services, mock external APIs
- **Speed**: Orta
- **Purpose**: Component integration validation

### Performance Tests
- **Scope**: Performance kritik operasyonlar
- **Dependencies**: Real implementations
- **Speed**: Yavaş
- **Purpose**: Performance regression detection

## 📋 Test Best Practices

### 1. Test Organization
- Clear test names (test_should_do_something_when_condition)
- One assertion per test
- Arrange-Act-Assert pattern
- Independent tests (no shared state)

### 2. Test Data Management
- Use builders for complex test data
- Prefer factories over hardcoded values
- Clean up test data after tests
- Use realistic but deterministic data

### 3. Async Testing
- Use #[tokio::test] for async tests
- Use #[serial] for database tests
- Proper error handling in tests
- Test both success and failure cases

### 4. Performance Testing
- Baseline measurements
- Regression detection
- Resource usage monitoring
- Realistic load patterns

## 🔧 Test Configuration

### Environment Variables
```bash
# Test database URL
DATABASE_URL=postgresql://test_user:test_pass@localhost/test_db

# JWT secrets for testing
JWT_SECRET=test_secret_key_for_testing
HASHING_SECRET=test_hashing_secret
```

### Test Dependencies (Cargo.toml)
```toml
[dev-dependencies]
tokio-test = \"0.4\"
criterion = { version = \"0.5\", features = [\"html_reports\"] }
serial_test = \"3.0\"
fake = { version = \"2.9\", features = [\"derive\", \"chrono\"] }
proptest = \"1.0\"
mockall = \"0.12\"
testcontainers = { version = \"0.20\", features = [\"blocking\"] }
tower-test = \"0.4\"
reqwest = { version = \"0.12\", features = [\"json\"] }
temp-env = \"0.3\"
assert_matches = \"1.5\"
```

## 🎯 Test Metrics

Mevcut test coverage'ı:

- **Total Tests**: 44 test
- **Unit Tests**: 34 test
- **Integration Tests**: 10 test  
- **Benchmark Tests**: 4 benchmark
- **Test Files**: 4 dosya

### Test Performance
- JWT Token Generation: ~50μs
- JWT Token Verification: ~30μs
- Model Creation: ~1μs
- Request Processing: ~100μs

## 🚨 Troubleshooting

### Common Issues

1. **Database Connection Issues**
   ```bash
   # Test database'in çalıştığını kontrol et
   psql -h localhost -U test_user -d test_db
   ```

2. **Test Failures**
   ```bash
   # Verbose output ile detaylı hata mesajları
   cargo test -- --nocapture
   ```

3. **Performance Test Issues**
   ```bash
   # Benchmark'ları tek tek çalıştır
   cargo bench --bench performance_tests
   ```

### Debug Mode

Test'leri debug mode'da çalıştırmak için:

```bash
# Debug bilgileri ile test çalıştır
RUST_LOG=debug cargo test -- --nocapture

# Specific test'i debug et
RUST_LOG=debug cargo test test_specific_function -- --nocapture --exact
```

## 🔄 CI/CD Integration

GitHub Actions, Jenkins veya diğer CI/CD sistemleri için:

```bash
# CI pipeline script
./scripts/test_runner.sh ci
```

Bu script şunları yapar:
1. Code linting (clippy + rustfmt)
2. Security audit (cargo-audit)  
3. All tests (unit + integration)
4. Coverage report generation

## 📈 Continuous Improvement

Test suite'ini sürekli geliştirmek için:

1. **Test Coverage Monitoring**: Her PR'da coverage azalmasın
2. **Performance Regression**: Benchmark'larda performance kaybı olmasın
3. **Test Parallelization**: Test çalışma süresini optimize et
4. **Flaky Test Detection**: Kararsız testleri tespit et ve düzelt

## 🤝 Contributing

Test'lere katkı sağlarken:

1. Her yeni feature için test ekleyin
2. Test coverage'ı %80'in üzerinde tutun
3. Performance kritik kod'lar için benchmark ekleyin
4. Test best practice'lerine uyun
5. Documentation'ı güncel tutun

---

Bu test infrastructure'ı production-ready ve industry best practice'lerine uygun olarak tasarlanmıştır. Herhangi bir soru veya iyileştirme önerisi için issue açabilirsiniz.