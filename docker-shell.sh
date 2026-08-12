cat > docker-shell.sh << 'EOF'
#!/bin/bash
docker run --rm -it \
    --device /dev/kvm \
    -v "$(pwd)":/build \
    trangorgeos-build \
    bash
EOF
chmod +x docker-shell.sh