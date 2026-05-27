# tp-llm-owen

This project provides an interactive command-line interface (CLI) written in Rust to communicate with a local vision-capable Large Language Model (LLM) hosted via LM Studio. It allows users to send prompts alongside images or PDFs to the local model and handle the output seamlessly.

## I - Prerequisites & LM Studio Setup

To reproduce and test this project, you need to set up LM Studio on your machine:

1. **Install LM Studio**: Download and install the software from [https://lmstudio.ai/](https://lmstudio.ai/).
2. **Download a Vision-Capable Model**:
   * Open LM Studio and navigate to the search tab.
   * Look for a multimodal/vision model (such as `gemma-4-e4b` or any compatible model like `llava`).
   * Download the model files.
3. **Start the Local API Server**:
   * Open your terminal or PowerShell and run the following command to spin up the server:
     ```powershell
     lms server start
     ```
   * Alternatively, go to the **Local Server** tab (developer icon) inside the LM Studio GUI and click **Start Server**. It will expose an OpenAI-compatible endpoint at `http://localhost:1234/v1/chat/completions`.

## II - Installation & Execution

This repository requires the Rust toolchain (Cargo) to be installed on your computer.

1. Clone or fork this repository:
   ```bash
   git clone <repository-url>
   cd tp-llm-owen
    ```
2. Build and run the application in release mode:

```powershell
cargo run --release
```

## III - How to Use & Testing

The user experience is interactive thanks to the inquire crate. When launching the tool, follow these steps:

1. **Select Prompt Source**: Choose whether you want to type your prompt directly in the Terminal or load it from a Text/CSV file.

2. **Provide File Path**: Provide the path to the image or PDF file you want to analyze.

3. **Choose Output Destination**: Choose to display the model's answer directly in the Terminal or save it into an external Output file (txt).

### Standard Test Case

A sample image is included at the root of the repository to easily verify that your application works perfectly:

- Test Image Path: assets/images/will-shrek.jpg

- Recommended Test Prompt: "Describe what you see in this image in detail and explain the context or humor behind it if applicable."

### Testing weather app

A sample weather test case is included to verify that the weather tool works correctly:

- Test Prompt: "What's the weather like in Paris?"
- Expected Output: The model should respond with the current weather conditions in Paris.
- Actual Output: The model responded with the current weather conditions in Paris.

>[!NOTE]
> Actually you can't ask for weather today, it's only for today.

## IV - Future Improvements

- Implement a native file picker to select assets (.png, .jpeg, .pdf) instead of entering manual text paths.

- Strip Markdown formatting elements from the terminal response to optimize text readability.
