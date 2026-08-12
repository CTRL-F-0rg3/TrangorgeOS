#!/bin/bash
echo "🔨 Budowanie obrazu Dockera dla TrangorgeOS..."
docker build -t trangorgeos-build .
echo "✅ Obraz zbudowany pomyślnie!"