

echo "Compiling Pitlane"
cargo build  

echo "Copying to target directory"
cp ./target/debug/pitlane_rs ../../

echo "Running Pitlane"
../../pitlane_rs $1 $2
