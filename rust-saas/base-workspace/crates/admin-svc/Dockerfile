# 使用预构建的基础镜像
FROM rust-base:latest as builder

WORKDIR /app

# 复制源代码
COPY ./crates/shared/ /app/crates/shared/
COPY ./base-workspace/crates/admin-svc/ /app/base-workspace/crates/admin-svc/

# 构建 admin-svc/base-workspace/
WORKDIR /app/base-workspace/
RUN cd crates/admin-svc && pwd &&  cargo build --release
# 使用运行时基础镜像
FROM rust-runtime:latest

# 复制编译好的二进制文件
COPY --from=builder /app/base-workspace/target/release/admin-svc .

# 启动服务
CMD ["./admin-svc"]
