root="../../../";
plugins_dir=$root"target/debug/plugins";
mkdir -p "$plugins_dir";
# Install ocl_cuckatoo
cd ocl_cuckatoo
cargo build
cd ..
if [ "$(uname)" = "Darwin" ]; then
	cp $root"target/debug/libocl_cuckatoo.dylib" $plugins_dir/ocl_cuckatoo.cuckooplugin
else
	cp $root"target/debug/libocl_cuckatoo.so" $plugins_dir/ocl_cuckatoo.cuckooplugin
fi

# Install ocl_cuckaroo
cd ocl_cuckaroo
cargo build
cd ..
if [ "$(uname)" = "Darwin" ]; then
	cp $root"target/debug/libocl_cuckaroo.dylib" $plugins_dir/ocl_cuckaroo.cuckooplugin
else
	cp $root"target/debug/libocl_cuckaroo.so" $plugins_dir/ocl_cuckaroo.cuckooplugin
fi
