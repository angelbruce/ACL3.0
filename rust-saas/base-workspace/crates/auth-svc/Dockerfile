# 使用预构建的基础镜像
FROM rust-base:latest as builder

WORKDIR /app

# 复制源代码
COPY crates/shared/src /app/crates/shared/src
COPY ./base-workspace/crates/auth-svc/src /app/base-workspace/crates/auth-svc/src

WORKDIR /app/base-workspace/
# 构建 auth-svc
RUN cd crates/auth-svc && cargo build --release

# 使用运行时基础镜像
FROM rust-runtime:latest

# 复制编译好的二进制文件
COPY --from=builder /app/workspace/target/release/auth-svc .

# 启动服务
CMD ["./auth-svc"]
