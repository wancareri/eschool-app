sed -i 's/\.height(0\.0)/\.height(0\.0)/g' src/pages/diary.rs
sed -i '/\.height(0\.0)/!b;n;s/        \.grow(),//' src/pages/diary.rs
