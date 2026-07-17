# 使用预构建的基础镜像
FROM rust-base:latest as builder

WORKDIR /app

# 复制源代码
COPY ./crates/shared/src /app/crates/shared/src
COPY ./base-workspace/crates/agent-svc/src /app/base-workspace/crates/agent-svc/src

WORKDIR /app/base-workspace
# 构建 agent-svc
RUN cd crates/agent-svc && cargo build --release

# 使用运行时基础镜像
FROM rust-runtime:latest

# 复制编译好的二进制文件
COPY --from=builder /app/base-workspace/target/release/agent-svc .

# 启动服务
CMD ["./agent-svc"]
