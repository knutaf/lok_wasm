rmdir /s /q %~dp0\www\dist
pushd %~dp0\www
npm run build
popd
