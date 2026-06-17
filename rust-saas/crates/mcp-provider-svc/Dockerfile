# 使用预构建的基础镜像
FROM rust-base:latest as builder

WORKDIR /app

# 复制源代码
COPY crates/shared/src crates/shared/src
COPY crates/mcp-provider-svc/src crates/mcp-provider-svc/src

# 构建 mcp-provider-svc
RUN cd crates/mcp-provider-svc && cargo build --release

# 使用运行时基础镜像
FROM rust-runtime:latest

# 复制编译好的二进制文件
COPY --from=builder /app/target/release/mcp-provider-svc .
ENV PORT 8088
EXPOSE $PORT
# 启动服务
CMD ["./mcp-provider-svc"]
