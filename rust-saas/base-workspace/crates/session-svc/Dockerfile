# 使用预构建的基础镜像
FROM rust-base:latest as builder

WORKDIR /app

# 复制源代码
COPY crates/shared/src crates/shared/src
COPY crates/session-svc/src /app/base-workspace/crates/session-svc/src

WORKDIR /app/base-workspace/
# 构建 session-svc
RUN cd crates/session-svc && cargo build --release

# 使用运行时基础镜像
FROM rust-runtime:latest

# 复制编译好的二进制文件
COPY --from=builder /app/base-workspace/target/release/session-svc .

# 启动服务
CMD ["./session-svc"]
