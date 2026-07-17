FROM rust-base:latest as builder

WORKDIR /app

# 复制补丁依赖与源码
COPY Cargo.toml .
COPY patches patches
COPY crates/shared/src crates/shared/src
COPY crates/vec-svc/src crates/vec-svc/src

# 构建 vec-svc
RUN cd crates/vec-svc && bash -c 'cargo check -p vec-svc 2>&1 | tee /tmp/check.log; exit ${PIPESTATUS[0]}' || (cat /tmp/check.log && exit 1)
RUN cd crates/vec-svc && cargo build --release -p vec-svc

# 运行时镜像
FROM rust-runtime:latest

WORKDIR /app

# 复制二进制文件
COPY --from=builder /app/target/release/vec-svc .

# 设置环境变量
ENV RUST_LOG=info

# 启动命令
CMD ["./vec-svc"]