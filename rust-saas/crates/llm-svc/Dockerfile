# 使用预构建的基础镜像
FROM rust-base:latest as builder

WORKDIR /app

# 复制源代码
COPY crates/shared/src crates/shared/src
COPY crates/llm-svc/src crates/llm-svc/src

# 构建 llm-svc
RUN cd crates/llm-svc && cargo build --release

# 使用运行时基础镜像
FROM rust-runtime:latest

# 复制编译好的二进制文件
COPY --from=builder /app/target/release/llm-svc .

# 启动服务
CMD ["./llm-svc"]
