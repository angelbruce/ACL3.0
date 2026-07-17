# 使用预构建的基础镜像
FROM rust-base:latest as builder

WORKDIR /app

# 复制 Cargo.toml 文件
COPY Cargo.toml Cargo.toml
COPY crates/shared/Cargo.toml crates/shared/Cargo.toml
COPY crates/flow-svc/Cargo.toml crates/flow-svc/Cargo.toml
COPY crates/extmacros/Cargo.toml crates/extmacros/Cargo.toml

# 复制 patches 目录（用于 shimmytok 补丁）
COPY patches patches

# 复制源代码
COPY crates/shared/src crates/shared/src
COPY crates/flow-svc/src crates/flow-svc/src
COPY crates/extmacros/src crates/extmacros/src  

# 构建 flow-svc
RUN cd crates/flow-svc && cargo build --release && cargo test

# 使用运行时基础镜像
FROM rust-runtime:latest

# 复制编译好的二进制文件
COPY --from=builder /app/target/release/flow-svc .

# 启动服务
CMD ["./flow-svc"]
