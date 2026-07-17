REM docker pull nvidia/cuda:12.4.0-runtime-ubuntu22.04
docker pull nvidia/cuda:12.4.0-runtime-ubuntu22.04
J:
cd J:\llama_cpp\project\ACL3.0M\github\ACL3.0\rust-saas
docker build -f Dockerfile.llama.cuda -t llama-cuda:latest .
echo "build llama-cuda:latest complete"