from openai import OpenAI

# Initialize client pointing to your custom provider (e.g., Local LLM, Together AI, Ollama)
client = OpenAI(
    base_url="ai.xytro.site",  # Replace with your endpoint
    api_key="valis_012260cca4c90f862ab8de34"               # Replace with your actual key
)

# Call the chat completions endpoint
response = client.chat.completions.create(
    model="xael-nano",                      # Specify the target model
    messages=[
        {"role": "system", "content": "You are a helpful assistant."},
        {"role": "user", "content": "Hello!"}
    ]
)

# Print the response content
print(response.choices[0].message.content)
