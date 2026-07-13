import re

def analyze_technical_paper(file_path):
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
    except FileNotFoundError:
        return "Error: File not found. Please check the path."

    # 1. Calculate Raw Word Count (Standard space-separated count)
    raw_words = content.split()
    total_raw_count = len(raw_words)

    # 2. Calculate Standard Academic Word Count
    # This regex finds text tokens that are purely alphabetical words.
    # It deliberately ignores standalone numbers, hex codes, symbols, and strict code tokens.
    academic_words = re.findall(r'\b[a-zA-Z]{2,}\b', content)
    total_academic_count = len(academic_words)

    # 3. Estimate Code & Benchmark Density
    difference = total_raw_count - total_academic_count
    density_percentage = (difference / total_raw_count) * 100 if total_raw_count > 0 else 0

    # Print Results
    print("=" * 50)
    print("         TECHNICAL PAPER WORD COUNT REPORT        ")
    print("=" * 50)
    print(f"Standard Academic Word Count : {total_academic_count:,} words")
    print(f"Raw Word Count (with Code/Data): {total_raw_count:,} words")
    print("-" * 50)
    print(f"Non-text Tokens (Code/Numbers) : {difference:,}")
    print(f"Code & Benchmark Data Density  : {density_percentage:.2f}%")
    print("=" * 50)
    print("\n[Insight] If your Academic Word Count is around 3,000-4,000,")
    print("your 22 citations are completely fine for submission!")
    print("=" * 50)

# How to use it:
# Replace 'your_paper.md' with your actual file path (works for .md, .txt, etc.)
analyze_technical_paper('doc/paper/TEF: A Portable Bytecode Format for Template Execution.md')
