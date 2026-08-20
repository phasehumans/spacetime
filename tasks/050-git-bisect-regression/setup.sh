#!/usr/bin/env bash
set -e

rm -rf /root/calculator
mkdir -p /root/calculator
cd /root/calculator

git init -b main
git config user.name "Spacetime"
git config user.email "test@spacetime.benchmark"

cat <<'EOF' > math_lib.py
def add(a, b): return a + b
def sub(a, b): return a - b
EOF

cat <<'EOF' > test.sh
#!/usr/bin/env bash
python3 -c "import math_lib; assert math_lib.add(2, 2) == 4 and math_lib.sub(5, 3) == 2"
EOF
chmod +x test.sh

git add .
git commit -m "initial commit: basic arithmetic"
git tag v1.0

# Commit 2: good
echo "# formatting" >> math_lib.py
git commit -am "chore: formatting"

# Commit 3: BAD commit
cat <<'EOF' > math_lib.py
def add(a, b): return a + b + 1
def sub(a, b): return a - b
EOF
git commit -am "refactor: performance optimization"
BAD_SHA=$(git rev-parse HEAD)
echo "$BAD_SHA" > /root/.expected_bad_commit

# Commit 4: after bad
echo "# more comments" >> math_lib.py
git commit -am "docs: update comments"

# Commit 5: after bad
echo "# final touches" >> math_lib.py
git commit -am "chore: release candidate"

rm -f /root/bad_commit.txt
