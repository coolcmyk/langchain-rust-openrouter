---
title: API Reference
subtitle: An overview of OpenRouter's API
headline: OpenRouter API Reference | Complete API Documentation
canonical-url: 'https://openrouter.ai/docs/api/reference/overview'
'og:site_name': OpenRouter Documentation
'og:title': OpenRouter API Reference - Complete Documentation
'og:description': >-
  Comprehensive guide to OpenRouter's API. Learn about request/response schemas,
  authentication, parameters, and integration with multiple AI model providers.
'og:image':
  type: url
  value: >-
    https://openrouter.ai/dynamic-og?title=OpenRouter%20API%20Reference&description=Comprehensive%20guide%20to%20OpenRouter's%20API.
'og:image:width': 1200
'og:image:height': 630
'twitter:card': summary_large_image
'twitter:site': '@OpenRouterAI'
noindex: false
nofollow: false
---

OpenRouter's request and response schemas are very similar to the OpenAI Chat API, with a few small differences. At a high level, **OpenRouter normalizes the schema across models and providers** so you only need to learn one.

## Requests

### Completions Request Format

Here is the request schema as a TypeScript type. This will be the body of your `POST` request to the `/api/v1/chat/completions` endpoint (see the [quick start](/docs/quickstart) above for an example).

For a complete list of parameters, see the [Parameters](/docs/api-reference/parameters).

<CodeGroup>

```typescript title="Request Schema"
// Definitions of subtypes are below
type Request = {
  // Either "messages" or "prompt" is required
  messages?: Message[];
  prompt?: string;

  // If "model" is unspecified, uses the user's default
  model?: string; // See "Supported Models" section

  // Allows to force the model to produce specific output format.
  // See models page and note on this docs page for which models support it.
  response_format?: { type: 'json_object' };

  stop?: string | string[];
  stream?: boolean; // Enable streaming

  // See LLM Parameters (openrouter.ai/docs/api/reference/parameters)
  max_tokens?: number; // Range: [1, context_length)
  temperature?: number; // Range: [0, 2]

  // Tool calling
  // Will be passed down as-is for providers implementing OpenAI's interface.
  // For providers with custom interfaces, we transform and map the properties.
  // Otherwise, we transform the tools into a YAML template. The model responds with an assistant message.
  // See models supporting tool calling: openrouter.ai/models?supported_parameters=tools
  tools?: Tool[];
  tool_choice?: ToolChoice;

  // Advanced optional parameters
  seed?: number; // Integer only
  top_p?: number; // Range: (0, 1]
  top_k?: number; // Range: [1, Infinity) Not available for OpenAI models
  frequency_penalty?: number; // Range: [-2, 2]
  presence_penalty?: number; // Range: [-2, 2]
  repetition_penalty?: number; // Range: (0, 2]
  logit_bias?: { [key: number]: number };
  top_logprobs: number; // Integer only
  min_p?: number; // Range: [0, 1]
  top_a?: number; // Range: [0, 1]

  // Reduce latency by providing the model with a predicted output
  // https://platform.openai.com/docs/guides/latency-optimization#use-predicted-outputs
  prediction?: { type: 'content'; content: string };

  // OpenRouter-only parameters
  // See "Prompt Transforms" section: openrouter.ai/docs/guides/features/message-transforms
  transforms?: string[];
  // See "Model Routing" section: openrouter.ai/docs/guides/features/model-routing
  models?: string[];
  route?: 'fallback';
  // See "Provider Routing" section: openrouter.ai/docs/guides/routing/provider-selection
  provider?: ProviderPreferences;
  user?: string; // A stable identifier for your end-users. Used to help detect and prevent abuse.
  
  // Debug options (streaming only)
  debug?: {
    echo_upstream_body?: boolean; // If true, returns the transformed request body sent to the provider
  };
};

// Subtypes:

type TextContent = {
  type: 'text';
  text: string;
};

type ImageContentPart = {
  type: 'image_url';
  image_url: {
    url: string; // URL or base64 encoded image data
    detail?: string; // Optional, defaults to "auto"
  };
};

type ContentPart = TextContent | ImageContentPart;

type Message =
  | {
      role: 'user' | 'assistant' | 'system';
      // ContentParts are only for the "user" role:
      content: string | ContentPart[];
      // If "name" is included, it will be prepended like this
      // for non-OpenAI models: `{name}: {content}`
      name?: string;
    }
  | {
      role: 'tool';
      content: string;
      tool_call_id: string;
      name?: string;
    };

type FunctionDescription = {
  description?: string;
  name: string;
  parameters: object; // JSON Schema object
};

type Tool = {
  type: 'function';
  function: FunctionDescription;
};

type ToolChoice =
  | 'none'
  | 'auto'
  | {
      type: 'function';
      function: {
        name: string;
      };
    };
```

</CodeGroup>

The `response_format` parameter ensures you receive a structured response from the LLM. The parameter is only supported by OpenAI models, Nitro models, and some others - check the providers on the model page on openrouter.ai/models to see if it's supported, and set `require_parameters` to true in your Provider Preferences. See [Provider Routing](/docs/features/provider-routing)

### Headers

OpenRouter allows you to specify some optional headers to identify your app and make it discoverable to users on our site.

- `HTTP-Referer`: Identifies your app on openrouter.ai
- `X-Title`: Sets/modifies your app's title

<CodeGroup>

```typescript title="TypeScript"
fetch('https://openrouter.ai/api/v1/chat/completions', {
  method: 'POST',
  headers: {
    Authorization: 'Bearer <OPENROUTER_API_KEY>',
    'HTTP-Referer': '<YOUR_SITE_URL>', // Optional. Site URL for rankings on openrouter.ai.
    'X-Title': '<YOUR_SITE_NAME>', // Optional. Site title for rankings on openrouter.ai.
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    model: 'openai/gpt-5.2',
    messages: [
      {
        role: 'user',
        content: 'What is the meaning of life?',
      },
    ],
  }),
});
```

</CodeGroup>

<Info title='Model routing'>
  If the `model` parameter is omitted, the user or payer's default is used.
  Otherwise, remember to select a value for `model` from the [supported
  models](/models) or [API](/api/v1/models), and include the organization
  prefix. OpenRouter will select the least expensive and best GPUs available to
  serve the request, and fall back to other providers or GPUs if it receives a
  5xx response code or if you are rate-limited.
</Info>

<Info title='Streaming'>
  [Server-Sent Events
  (SSE)](https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events#event_stream_format)
  are supported as well, to enable streaming _for all models_. Simply send
  `stream: true` in your request body. The SSE stream will occasionally contain
  a "comment" payload, which you should ignore (noted below).
</Info>

<Info title='Non-standard parameters'>
  If the chosen model doesn't support a request parameter (such as `logit_bias`
  in non-OpenAI models, or `top_k` for OpenAI), then the parameter is ignored.
  The rest are forwarded to the underlying model API.
</Info>

### Assistant Prefill

OpenRouter supports asking models to complete a partial response. This can be useful for guiding models to respond in a certain way.

To use this features, simply include a message with `role: "assistant"` at the end of your `messages` array.

<CodeGroup>

```typescript title="TypeScript"
fetch('https://openrouter.ai/api/v1/chat/completions', {
  method: 'POST',
  headers: {
    Authorization: 'Bearer <OPENROUTER_API_KEY>',
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    model: 'openai/gpt-5.2',
    messages: [
      { role: 'user', content: 'What is the meaning of life?' },
      { role: 'assistant', content: "I'm not sure, but my best guess is" },
    ],
  }),
});
```

</CodeGroup>

## Responses

### CompletionsResponse Format

OpenRouter normalizes the schema across models and providers to comply with the [OpenAI Chat API](https://platform.openai.com/docs/api-reference/chat).

This means that `choices` is always an array, even if the model only returns one completion. Each choice will contain a `delta` property if a stream was requested and a `message` property otherwise. This makes it easier to use the same code for all models.

Here's the response schema as a TypeScript type:

```typescript TypeScript
// Definitions of subtypes are below
type Response = {
  id: string;
  // Depending on whether you set "stream" to "true" and
  // whether you passed in "messages" or a "prompt", you
  // will get a different output shape
  choices: (NonStreamingChoice | StreamingChoice | NonChatChoice)[];
  created: number; // Unix timestamp
  model: string;
  object: 'chat.completion' | 'chat.completion.chunk';

  system_fingerprint?: string; // Only present if the provider supports it

  // Usage data is always returned for non-streaming.
  // When streaming, you will get one usage object at
  // the end accompanied by an empty choices array.
  usage?: ResponseUsage;
};
```

```typescript
// If the provider returns usage, we pass it down
// as-is. Otherwise, we count using the GPT-4 tokenizer.

type ResponseUsage = {
  /** Including images and tools if any */
  prompt_tokens: number;
  /** The tokens generated */
  completion_tokens: number;
  /** Sum of the above two fields */
  total_tokens: number;
};
```

```typescript
// Subtypes:
type NonChatChoice = {
  finish_reason: string | null;
  text: string;
  error?: ErrorResponse;
};

type NonStreamingChoice = {
  finish_reason: string | null;
  native_finish_reason: string | null;
  message: {
    content: string | null;
    role: string;
    tool_calls?: ToolCall[];
  };
  error?: ErrorResponse;
};

type StreamingChoice = {
  finish_reason: string | null;
  native_finish_reason: string | null;
  delta: {
    content: string | null;
    role?: string;
    tool_calls?: ToolCall[];
  };
  error?: ErrorResponse;
};

type ErrorResponse = {
  code: number; // See "Error Handling" section
  message: string;
  metadata?: Record<string, unknown>; // Contains additional error information such as provider details, the raw error message, etc.
};

type ToolCall = {
  id: string;
  type: 'function';
  function: FunctionCall;
};
```

Here's an example:

```json
{
  "id": "gen-xxxxxxxxxxxxxx",
  "choices": [
    {
      "finish_reason": "stop", // Normalized finish_reason
      "native_finish_reason": "stop", // The raw finish_reason from the provider
      "message": {
        // will be "delta" if streaming
        "role": "assistant",
        "content": "Hello there!"
      }
    }
  ],
  "usage": {
    "prompt_tokens": 0,
    "completion_tokens": 4,
    "total_tokens": 4
  },
  "model": "openai/gpt-3.5-turbo" // Could also be "anthropic/claude-2.1", etc, depending on the "model" that ends up being used
}
```

### Finish Reason

OpenRouter normalizes each model's `finish_reason` to one of the following values: `tool_calls`, `stop`, `length`, `content_filter`, `error`.

Some models and providers may have additional finish reasons. The raw finish_reason string returned by the model is available via the `native_finish_reason` property.

### Querying Cost and Stats

The token counts that are returned in the completions API response are **not** counted via the model's native tokenizer. Instead it uses a normalized, model-agnostic count (accomplished via the GPT4o tokenizer). This is because some providers do not reliably return native token counts. This behavior is becoming more rare, however, and we may add native token counts to the response object in the future.

Credit usage and model pricing are based on the **native** token counts (not the 'normalized' token counts returned in the API response).

For precise token accounting using the model's native tokenizer, you can retrieve the full generation information via the `/api/v1/generation` endpoint.

You can use the returned `id` to query for the generation stats (including token counts and cost) after the request is complete. This is how you can get the cost and tokens for _all models and requests_, streaming and non-streaming.

<CodeGroup>

```typescript title="Query Generation Stats"
const generation = await fetch(
  'https://openrouter.ai/api/v1/generation?id=$GENERATION_ID',
  { headers },
);

const stats = await generation.json();
```

</CodeGroup>

Please see the [Generation](/docs/api-reference/get-a-generation) API reference for the full response shape.

Note that token counts are also available in the `usage` field of the response body for non-streaming completions.



---
title: Streaming
headline: API Streaming | Real-time Model Responses in OpenRouter
canonical-url: 'https://openrouter.ai/docs/api/reference/streaming'
'og:site_name': OpenRouter Documentation
'og:title': API Streaming - Real-time Model Response Integration
'og:description': >-
  Learn how to implement streaming responses with OpenRouter's API. Complete
  guide to Server-Sent Events (SSE) and real-time model outputs.
'og:image':
  type: url
  value: >-
    https://openrouter.ai/dynamic-og?title=API%20Streaming&description=Real-time%20model%20response%20streaming
'og:image:width': 1200
'og:image:height': 630
'twitter:card': summary_large_image
'twitter:site': '@OpenRouterAI'
noindex: false
nofollow: false
---

import { API_KEY_REF, Model } from '../../../imports/constants';

The OpenRouter API allows streaming responses from _any model_. This is useful for building chat interfaces or other applications where the UI should update as the model generates the response.

To enable streaming, you can set the `stream` parameter to `true` in your request. The model will then stream the response to the client in chunks, rather than returning the entire response at once.

Here is an example of how to stream a response, and process it:

<Template data={{
  API_KEY_REF,
  MODEL: Model.GPT_4_Omni
}}>

<CodeGroup>

```typescript title="TypeScript SDK"
import { OpenRouter } from '@openrouter/sdk';

const openRouter = new OpenRouter({
  apiKey: '{{API_KEY_REF}}',
});

const question = 'How would you build the tallest building ever?';

const stream = await openRouter.chat.send({
  model: '{{MODEL}}',
  messages: [{ role: 'user', content: question }],
  stream: true,
  streamOptions: { includeUsage: true }
});

for await (const chunk of stream) {
  const content = chunk.choices?.[0]?.delta?.content;
  if (content) {
    console.log(content);
  }

  // Final chunk includes usage stats
  if (chunk.usage) {
    console.log('Usage:', chunk.usage);
  }
}
```

```python Python
import requests
import json

question = "How would you build the tallest building ever?"

url = "https://openrouter.ai/api/v1/chat/completions"
headers = {
  "Authorization": f"Bearer {{API_KEY_REF}}",
  "Content-Type": "application/json"
}

payload = {
  "model": "{{MODEL}}",
  "messages": [{"role": "user", "content": question}],
  "stream": True
}

buffer = ""
with requests.post(url, headers=headers, json=payload, stream=True) as r:
  for chunk in r.iter_content(chunk_size=1024, decode_unicode=True):
    buffer += chunk
    while True:
      try:
        # Find the next complete SSE line
        line_end = buffer.find('\n')
        if line_end == -1:
          break

        line = buffer[:line_end].strip()
        buffer = buffer[line_end + 1:]

        if line.startswith('data: '):
          data = line[6:]
          if data == '[DONE]':
            break

          try:
            data_obj = json.loads(data)
            content = data_obj["choices"][0]["delta"].get("content")
            if content:
              print(content, end="", flush=True)
          except json.JSONDecodeError:
            pass
      except Exception:
        break
```

```typescript title="TypeScript (fetch)"
const question = 'How would you build the tallest building ever?';
const response = await fetch('https://openrouter.ai/api/v1/chat/completions', {
  method: 'POST',
  headers: {
    Authorization: `Bearer ${API_KEY_REF}`,
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    model: '{{MODEL}}',
    messages: [{ role: 'user', content: question }],
    stream: true,
  }),
});

const reader = response.body?.getReader();
if (!reader) {
  throw new Error('Response body is not readable');
}

const decoder = new TextDecoder();
let buffer = '';

try {
  while (true) {
    const { done, value } = await reader.read();
    if (done) break;

    // Append new chunk to buffer
    buffer += decoder.decode(value, { stream: true });

    // Process complete lines from buffer
    while (true) {
      const lineEnd = buffer.indexOf('\n');
      if (lineEnd === -1) break;

      const line = buffer.slice(0, lineEnd).trim();
      buffer = buffer.slice(lineEnd + 1);

      if (line.startsWith('data: ')) {
        const data = line.slice(6);
        if (data === '[DONE]') break;

        try {
          const parsed = JSON.parse(data);
          const content = parsed.choices[0].delta.content;
          if (content) {
            console.log(content);
          }
        } catch (e) {
          // Ignore invalid JSON
        }
      }
    }
  }
} finally {
  reader.cancel();
}
```

</CodeGroup>
</Template>

### Additional Information

For SSE (Server-Sent Events) streams, OpenRouter occasionally sends comments to prevent connection timeouts. These comments look like:

```text
: OPENROUTER PROCESSING
```

Comment payload can be safely ignored per the [SSE specs](https://html.spec.whatwg.org/multipage/server-sent-events.html#event-stream-interpretation). However, you can leverage it to improve UX as needed, e.g. by showing a dynamic loading indicator.

Some SSE client implementations might not parse the payload according to spec, which leads to an uncaught error when you `JSON.stringify` the non-JSON payloads. We recommend the following clients:

- [eventsource-parser](https://github.com/rexxars/eventsource-parser)
- [OpenAI SDK](https://www.npmjs.com/package/openai)
- [Vercel AI SDK](https://www.npmjs.com/package/ai)

### Stream Cancellation

Streaming requests can be cancelled by aborting the connection. For supported providers, this immediately stops model processing and billing.

<Accordion title="Provider Support">

**Supported**

- OpenAI, Azure, Anthropic
- Fireworks, Mancer, Recursal
- AnyScale, Lepton, OctoAI
- Novita, DeepInfra, Together
- Cohere, Hyperbolic, Infermatic
- Avian, XAI, Cloudflare
- SFCompute, Nineteen, Liquid
- Friendli, Chutes, DeepSeek

**Not Currently Supported**

- AWS Bedrock, Groq, Modal
- Google, Google AI Studio, Minimax
- HuggingFace, Replicate, Perplexity
- Mistral, AI21, Featherless
- Lynn, Lambda, Reflection
- SambaNova, Inflection, ZeroOneAI
- AionLabs, Alibaba, Nebius
- Kluster, Targon, InferenceNet

</Accordion>

To implement stream cancellation:

<Template data={{
  API_KEY_REF,
  MODEL: Model.GPT_4_Omni
}}>

<CodeGroup>

```typescript title="TypeScript SDK"
import { OpenRouter } from '@openrouter/sdk';

const openRouter = new OpenRouter({
  apiKey: '{{API_KEY_REF}}',
});

const controller = new AbortController();

try {
  const stream = await openRouter.chat.send({
    model: '{{MODEL}}',
    messages: [{ role: 'user', content: 'Write a story' }],
    stream: true,
  }, {
    signal: controller.signal,
  });

  for await (const chunk of stream) {
    const content = chunk.choices?.[0]?.delta?.content;
    if (content) {
      console.log(content);
    }
  }
} catch (error) {
  if (error.name === 'AbortError') {
    console.log('Stream cancelled');
  } else {
    throw error;
  }
}

// To cancel the stream:
controller.abort();
```

```python Python
import requests
from threading import Event, Thread

def stream_with_cancellation(prompt: str, cancel_event: Event):
    with requests.Session() as session:
        response = session.post(
            "https://openrouter.ai/api/v1/chat/completions",
            headers={"Authorization": f"Bearer {{API_KEY_REF}}"},
            json={"model": "{{MODEL}}", "messages": [{"role": "user", "content": prompt}], "stream": True},
            stream=True
        )

        try:
            for line in response.iter_lines():
                if cancel_event.is_set():
                    response.close()
                    return
                if line:
                    print(line.decode(), end="", flush=True)
        finally:
            response.close()

# Example usage:
cancel_event = Event()
stream_thread = Thread(target=lambda: stream_with_cancellation("Write a story", cancel_event))
stream_thread.start()

# To cancel the stream:
cancel_event.set()
```

```typescript title="TypeScript (fetch)"
const controller = new AbortController();

try {
  const response = await fetch(
    'https://openrouter.ai/api/v1/chat/completions',
    {
      method: 'POST',
      headers: {
        Authorization: `Bearer ${{{API_KEY_REF}}}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        model: '{{MODEL}}',
        messages: [{ role: 'user', content: 'Write a story' }],
        stream: true,
      }),
      signal: controller.signal,
    },
  );

  // Process the stream...
} catch (error) {
  if (error.name === 'AbortError') {
    console.log('Stream cancelled');
  } else {
    throw error;
  }
}

// To cancel the stream:
controller.abort();
```

</CodeGroup>
</Template>

<Warning>
  Cancellation only works for streaming requests with supported providers. For
  non-streaming requests or unsupported providers, the model will continue
  processing and you will be billed for the complete response.
</Warning>

### Handling Errors During Streaming

OpenRouter handles errors differently depending on when they occur during the streaming process:

#### Errors Before Any Tokens Are Sent

If an error occurs before any tokens have been streamed to the client, OpenRouter returns a standard JSON error response with the appropriate HTTP status code. This follows the standard error format:

```json
{
  "error": {
    "code": 400,
    "message": "Invalid model specified"
  }
}
```

Common HTTP status codes include:
- **400**: Bad Request (invalid parameters)
- **401**: Unauthorized (invalid API key)
- **402**: Payment Required (insufficient credits)
- **429**: Too Many Requests (rate limited)
- **502**: Bad Gateway (provider error)
- **503**: Service Unavailable (no available providers)

#### Errors After Tokens Have Been Sent (Mid-Stream)

If an error occurs after some tokens have already been streamed to the client, OpenRouter cannot change the HTTP status code (which is already 200 OK). Instead, the error is sent as a Server-Sent Event (SSE) with a unified structure:

```text
data: {"id":"cmpl-abc123","object":"chat.completion.chunk","created":1234567890,"model":"gpt-3.5-turbo","provider":"openai","error":{"code":"server_error","message":"Provider disconnected unexpectedly"},"choices":[{"index":0,"delta":{"content":""},"finish_reason":"error"}]}
```

Key characteristics of mid-stream errors:
- The error appears at the **top level** alongside standard response fields (id, object, created, etc.)
- A `choices` array is included with `finish_reason: "error"` to properly terminate the stream
- The HTTP status remains 200 OK since headers were already sent
- The stream is terminated after this unified error event

#### Code Examples

Here's how to properly handle both types of errors in your streaming implementation:

<Template data={{
  API_KEY_REF,
  MODEL: Model.GPT_4_Omni
}}>

<CodeGroup>

```typescript title="TypeScript SDK"
import { OpenRouter } from '@openrouter/sdk';

const openRouter = new OpenRouter({
  apiKey: '{{API_KEY_REF}}',
});

async function streamWithErrorHandling(prompt: string) {
  try {
    const stream = await openRouter.chat.send({
      model: '{{MODEL}}',
      messages: [{ role: 'user', content: prompt }],
      stream: true,
    });

    for await (const chunk of stream) {
      // Check for errors in chunk
      if ('error' in chunk) {
        console.error(`Stream error: ${chunk.error.message}`);
        if (chunk.choices?.[0]?.finish_reason === 'error') {
          console.log('Stream terminated due to error');
        }
        return;
      }

      // Process normal content
      const content = chunk.choices?.[0]?.delta?.content;
      if (content) {
        console.log(content);
      }
    }
  } catch (error) {
    // Handle pre-stream errors
    console.error(`Error: ${error.message}`);
  }
}
```

```python Python
import requests
import json

async def stream_with_error_handling(prompt):
    response = requests.post(
        'https://openrouter.ai/api/v1/chat/completions',
        headers={'Authorization': f'Bearer {{API_KEY_REF}}'},
        json={
            'model': '{{MODEL}}',
            'messages': [{'role': 'user', 'content': prompt}],
            'stream': True
        },
        stream=True
    )

    # Check initial HTTP status for pre-stream errors
    if response.status_code != 200:
        error_data = response.json()
        print(f"Error: {error_data['error']['message']}")
        return

    # Process stream and handle mid-stream errors
    for line in response.iter_lines():
        if line:
            line_text = line.decode('utf-8')
            if line_text.startswith('data: '):
                data = line_text[6:]
                if data == '[DONE]':
                    break

                try:
                    parsed = json.loads(data)

                    # Check for mid-stream error
                    if 'error' in parsed:
                        print(f"Stream error: {parsed['error']['message']}")
                        # Check finish_reason if needed
                        if parsed.get('choices', [{}])[0].get('finish_reason') == 'error':
                            print("Stream terminated due to error")
                        break

                    # Process normal content
                    content = parsed['choices'][0]['delta'].get('content')
                    if content:
                        print(content, end='', flush=True)

                except json.JSONDecodeError:
                    pass
```

```typescript title="TypeScript (fetch)"
async function streamWithErrorHandling(prompt: string) {
  const response = await fetch(
    'https://openrouter.ai/api/v1/chat/completions',
    {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${{{API_KEY_REF}}}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        model: '{{MODEL}}',
        messages: [{ role: 'user', content: prompt }],
        stream: true,
      }),
    }
  );

  // Check initial HTTP status for pre-stream errors
  if (!response.ok) {
    const error = await response.json();
    console.error(`Error: ${error.error.message}`);
    return;
  }

  const reader = response.body?.getReader();
  if (!reader) throw new Error('No response body');

  const decoder = new TextDecoder();
  let buffer = '';

  try {
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;

      buffer += decoder.decode(value, { stream: true });

      while (true) {
        const lineEnd = buffer.indexOf('\n');
        if (lineEnd === -1) break;

        const line = buffer.slice(0, lineEnd).trim();
        buffer = buffer.slice(lineEnd + 1);

        if (line.startsWith('data: ')) {
          const data = line.slice(6);
          if (data === '[DONE]') return;

          try {
            const parsed = JSON.parse(data);

            // Check for mid-stream error
            if (parsed.error) {
              console.error(`Stream error: ${parsed.error.message}`);
              // Check finish_reason if needed
              if (parsed.choices?.[0]?.finish_reason === 'error') {
                console.log('Stream terminated due to error');
              }
              return;
            }

            // Process normal content
            const content = parsed.choices[0].delta.content;
            if (content) {
              console.log(content);
            }
          } catch (e) {
            // Ignore parsing errors
          }
        }
      }
    }
  } finally {
    reader.cancel();
  }
}
```

</CodeGroup>
</Template>

#### API-Specific Behavior

Different API endpoints may handle streaming errors slightly differently:

- **OpenAI Chat Completions API**: Returns `ErrorResponse` directly if no chunks were processed, or includes error information in the response if some chunks were processed
- **OpenAI Responses API**: May transform certain error codes (like `context_length_exceeded`) into a successful response with `finish_reason: "length"` instead of treating them as errors


---
title: Embeddings
subtitle: Generate vector embeddings from text
headline: Embeddings API | Convert Text to Vector Representations with OpenRouter
canonical-url: 'https://openrouter.ai/docs/api/reference/embeddings'
'og:site_name': OpenRouter Documentation
'og:title': Embeddings API - Generate Vector Embeddings from Text
'og:description': >-
  Generate vector embeddings from text using OpenRouter's unified embeddings
  API. Access multiple embedding models from different providers with a single
  interface.
'og:image':
  type: url
  value: >-
    https://openrouter.ai/dynamic-og?title=Embeddings%20API&description=Generate%20Vector%20Embeddings%20from%20Text
'og:image:width': 1200
'og:image:height': 630
'twitter:card': summary_large_image
'twitter:site': '@OpenRouterAI'
noindex: false
nofollow: false
---

import { API_KEY_REF } from '../../../imports/constants';

Embeddings are numerical representations of text that capture semantic meaning. They convert text into vectors (arrays of numbers) that can be used for various machine learning tasks. OpenRouter provides a unified API to access embedding models from multiple providers.

## What are Embeddings?

Embeddings transform text into high-dimensional vectors where semantically similar texts are positioned closer together in vector space. For example, "cat" and "kitten" would have similar embeddings, while "cat" and "airplane" would be far apart.

These vector representations enable machines to understand relationships between pieces of text, making them essential for many AI applications.

## Common Use Cases

Embeddings are used in a wide variety of applications:

**RAG (Retrieval-Augmented Generation)**: Build RAG systems that retrieve relevant context from a knowledge base before generating answers. Embeddings help find the most relevant documents to include in the LLM's context.

**Semantic Search**: Convert documents and queries into embeddings, then find the most relevant documents by comparing vector similarity. This provides more accurate results than traditional keyword matching because it understands meaning rather than just matching words.

**Recommendation Systems**: Generate embeddings for items (products, articles, movies) and user preferences to recommend similar items. By comparing embedding vectors, you can find items that are semantically related even if they don't share obvious keywords.

**Clustering and Classification**: Group similar documents together or classify text into categories by analyzing embedding patterns. Documents with similar embeddings likely belong to the same topic or category.

**Duplicate Detection**: Identify duplicate or near-duplicate content by comparing embedding similarity. This works even when text is paraphrased or reworded.

**Anomaly Detection**: Detect unusual or outlier content by identifying embeddings that are far from typical patterns in your dataset.

## How to Use Embeddings

### Basic Request

To generate embeddings, send a POST request to `/embeddings` with your text input and chosen model:

<Template data={{
  API_KEY_REF,
  MODEL: 'openai/text-embedding-3-small'
}}>
<CodeGroup>

```typescript title="TypeScript SDK"
import { OpenRouter } from '@openrouter/sdk';

const openRouter = new OpenRouter({
  apiKey: '{{API_KEY_REF}}',
});

const response = await openRouter.embeddings.generate({
  model: '{{MODEL}}',
  input: 'The quick brown fox jumps over the lazy dog',
});

console.log(response.data[0].embedding);
```

```python title="Python"
import requests

response = requests.post(
  "https://openrouter.ai/api/v1/embeddings",
  headers={
    "Authorization": f"Bearer {{API_KEY_REF}}",
    "Content-Type": "application/json",
  },
  json={
    "model": "{{MODEL}}",
    "input": "The quick brown fox jumps over the lazy dog"
  }
)

data = response.json()
embedding = data["data"][0]["embedding"]
print(f"Embedding dimension: {len(embedding)}")
```

```typescript title="TypeScript (fetch)"
const response = await fetch('https://openrouter.ai/api/v1/embeddings', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer {{API_KEY_REF}}',
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    model: '{{MODEL}}',
    input: 'The quick brown fox jumps over the lazy dog',
  }),
});

const data = await response.json();
const embedding = data.data[0].embedding;
console.log(`Embedding dimension: ${embedding.length}`);
```

```shell title="Shell"
curl https://openrouter.ai/api/v1/embeddings \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $OPENROUTER_API_KEY" \
  -d '{
    "model": "{{MODEL}}",
    "input": "The quick brown fox jumps over the lazy dog"
  }'
```

</CodeGroup>
</Template>

### Batch Processing

You can generate embeddings for multiple texts in a single request by passing an array of strings:

<Template data={{
  API_KEY_REF,
  MODEL: 'openai/text-embedding-3-small'
}}>
<CodeGroup>

```typescript title="TypeScript SDK"
import { OpenRouter } from '@openrouter/sdk';

const openRouter = new OpenRouter({
  apiKey: '{{API_KEY_REF}}',
});

const response = await openRouter.embeddings.generate({
  model: '{{MODEL}}',
  input: [
    'Machine learning is a subset of artificial intelligence',
    'Deep learning uses neural networks with multiple layers',
    'Natural language processing enables computers to understand text'
  ],
});

// Process each embedding
response.data.forEach((item, index) => {
  console.log(`Embedding ${index}: ${item.embedding.length} dimensions`);
});
```

```python title="Python"
import requests

response = requests.post(
  "https://openrouter.ai/api/v1/embeddings",
  headers={
    "Authorization": f"Bearer {{API_KEY_REF}}",
    "Content-Type": "application/json",
  },
  json={
    "model": "{{MODEL}}",
    "input": [
      "Machine learning is a subset of artificial intelligence",
      "Deep learning uses neural networks with multiple layers",
      "Natural language processing enables computers to understand text"
    ]
  }
)

data = response.json()
for i, item in enumerate(data["data"]):
  print(f"Embedding {i}: {len(item['embedding'])} dimensions")
```

```typescript title="TypeScript (fetch)"
const response = await fetch('https://openrouter.ai/api/v1/embeddings', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer {{API_KEY_REF}}',
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    model: '{{MODEL}}',
    input: [
      'Machine learning is a subset of artificial intelligence',
      'Deep learning uses neural networks with multiple layers',
      'Natural language processing enables computers to understand text'
    ],
  }),
});

const data = await response.json();
data.data.forEach((item, index) => {
  console.log(`Embedding ${index}: ${item.embedding.length} dimensions`);
});
```

```shell title="Shell"
curl https://openrouter.ai/api/v1/embeddings \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $OPENROUTER_API_KEY" \
  -d '{
    "model": "{{MODEL}}",
    "input": [
      "Machine learning is a subset of artificial intelligence",
      "Deep learning uses neural networks with multiple layers",
      "Natural language processing enables computers to understand text"
    ]
  }'
```

</CodeGroup>
</Template>

## API Reference

For detailed information about request parameters, response format, and all available options, see the [Embeddings API Reference](/docs/api-reference/embeddings/create-embeddings).

## Available Models

OpenRouter provides access to various embedding models from different providers. You can view all available embedding models at:

[https://openrouter.ai/models?fmt=cards&output_modalities=embeddings](https://openrouter.ai/models?fmt=cards&output_modalities=embeddings)

To list all available embedding models programmatically:

<Template data={{
  API_KEY_REF
}}>
<CodeGroup>

```typescript title="TypeScript SDK"
import { OpenRouter } from '@openrouter/sdk';

const openRouter = new OpenRouter({
  apiKey: '{{API_KEY_REF}}',
});

const models = await openRouter.embeddings.listModels();
console.log(models.data);
```

```python title="Python"
import requests

response = requests.get(
  "https://openrouter.ai/api/v1/embeddings/models",
  headers={
    "Authorization": f"Bearer {{API_KEY_REF}}",
  }
)

models = response.json()
for model in models["data"]:
  print(f"{model['id']}: {model.get('context_length', 'N/A')} tokens")
```

```typescript title="TypeScript (fetch)"
const response = await fetch('https://openrouter.ai/api/v1/embeddings/models', {
  headers: {
    'Authorization': 'Bearer {{API_KEY_REF}}',
  },
});

const models = await response.json();
console.log(models.data);
```

```shell title="Shell"
curl https://openrouter.ai/api/v1/embeddings/models \
  -H "Authorization: Bearer $OPENROUTER_API_KEY"
```

</CodeGroup>
</Template>

## Practical Example: Semantic Search

Here's a complete example of building a semantic search system using embeddings:

<Template data={{
  API_KEY_REF,
  MODEL: 'openai/text-embedding-3-small'
}}>
<CodeGroup>

```typescript title="TypeScript SDK"
import { OpenRouter } from '@openrouter/sdk';

const openRouter = new OpenRouter({
  apiKey: '{{API_KEY_REF}}',
});

// Sample documents
const documents = [
  "The cat sat on the mat",
  "Dogs are loyal companions",
  "Python is a programming language",
  "Machine learning models require training data",
  "The weather is sunny today"
];

// Function to calculate cosine similarity
function cosineSimilarity(a: number[], b: number[]): number {
  const dotProduct = a.reduce((sum, val, i) => sum + val * b[i], 0);
  const magnitudeA = Math.sqrt(a.reduce((sum, val) => sum + val * val, 0));
  const magnitudeB = Math.sqrt(b.reduce((sum, val) => sum + val * val, 0));
  return dotProduct / (magnitudeA * magnitudeB);
}

async function semanticSearch(query: string, documents: string[]) {
  // Generate embeddings for all documents and the query
  const response = await openRouter.embeddings.generate({
    model: '{{MODEL}}',
    input: [query, ...documents],
  });

  const queryEmbedding = response.data[0].embedding;
  const docEmbeddings = response.data.slice(1);

  // Calculate similarity scores
  const results = documents.map((doc, i) => ({
    document: doc,
    similarity: cosineSimilarity(
      queryEmbedding as number[],
      docEmbeddings[i].embedding as number[]
    ),
  }));

  // Sort by similarity (highest first)
  results.sort((a, b) => b.similarity - a.similarity);

  return results;
}

// Search for documents related to pets
const results = await semanticSearch("pets and animals", documents);
console.log("Search results:");
results.forEach((result, i) => {
  console.log(`${i + 1}. ${result.document} (similarity: ${result.similarity.toFixed(4)})`);
});
```

```python title="Python"
import requests
import numpy as np

OPENROUTER_API_KEY = "{{API_KEY_REF}}"

# Sample documents
documents = [
  "The cat sat on the mat",
  "Dogs are loyal companions",
  "Python is a programming language",
  "Machine learning models require training data",
  "The weather is sunny today"
]

def cosine_similarity(a, b):
  """Calculate cosine similarity between two vectors"""
  dot_product = np.dot(a, b)
  magnitude_a = np.linalg.norm(a)
  magnitude_b = np.linalg.norm(b)
  return dot_product / (magnitude_a * magnitude_b)

def semantic_search(query, documents):
  """Perform semantic search using embeddings"""
  # Generate embeddings for query and all documents
  response = requests.post(
    "https://openrouter.ai/api/v1/embeddings",
    headers={
      "Authorization": f"Bearer {OPENROUTER_API_KEY}",
      "Content-Type": "application/json",
    },
    json={
      "model": "{{MODEL}}",
      "input": [query] + documents
    }
  )
  
  data = response.json()
  query_embedding = np.array(data["data"][0]["embedding"])
  doc_embeddings = [np.array(item["embedding"]) for item in data["data"][1:]]
  
  # Calculate similarity scores
  results = []
  for i, doc in enumerate(documents):
    similarity = cosine_similarity(query_embedding, doc_embeddings[i])
    results.append({"document": doc, "similarity": similarity})
  
  # Sort by similarity (highest first)
  results.sort(key=lambda x: x["similarity"], reverse=True)
  
  return results

# Search for documents related to pets
results = semantic_search("pets and animals", documents)
print("Search results:")
for i, result in enumerate(results):
  print(f"{i + 1}. {result['document']} (similarity: {result['similarity']:.4f})")
```

</CodeGroup>
</Template>

Expected output:
```
Search results:
1. Dogs are loyal companions (similarity: 0.8234)
2. The cat sat on the mat (similarity: 0.7891)
3. The weather is sunny today (similarity: 0.3456)
4. Machine learning models require training data (similarity: 0.2987)
5. Python is a programming language (similarity: 0.2654)
```

## Best Practices

**Choose the Right Model**: Different embedding models have different strengths. Smaller models (like qwen/qwen3-embedding-0.6b or openai/text-embedding-3-small) are faster and cheaper, while larger models (like openai/text-embedding-3-large) provide better quality. Test multiple models to find the best fit for your use case.

**Batch Your Requests**: When processing multiple texts, send them in a single request rather than making individual API calls. This reduces latency and costs.

**Cache Embeddings**: Embeddings for the same text are deterministic (they don't change). Store embeddings in a database or vector store to avoid regenerating them repeatedly.

**Normalize for Comparison**: When comparing embeddings, use cosine similarity rather than Euclidean distance. Cosine similarity is scale-invariant and works better for high-dimensional vectors.

**Consider Context Length**: Each model has a maximum input length (context window). Longer texts may need to be chunked or truncated. Check the model's specifications before processing long documents.

**Use Appropriate Chunking**: For long documents, split them into meaningful chunks (paragraphs, sections) rather than arbitrary character limits. This preserves semantic coherence.

## Provider Routing

You can control which providers serve your embedding requests using the `provider` parameter. This is useful for:

- Ensuring data privacy with specific providers
- Optimizing for cost or latency
- Using provider-specific features

Example with provider preferences:

```typescript
{
  "model": "openai/text-embedding-3-small",
  "input": "Your text here",
  "provider": {
    "order": ["openai", "azure"],
    "allow_fallbacks": true,
    "data_collection": "deny"
  }
}
```

For more information, see [Provider Routing](/docs/features/provider-routing).

## Error Handling

Common errors you may encounter:

**400 Bad Request**: Invalid input format or missing required parameters. Check that your `input` and `model` parameters are correctly formatted.

**401 Unauthorized**: Invalid or missing API key. Verify your API key is correct and included in the Authorization header.

**402 Payment Required**: Insufficient credits. Add credits to your OpenRouter account.

**404 Not Found**: The specified model doesn't exist or isn't available for embeddings. Check the model name and verify it's an embedding model.

**429 Too Many Requests**: Rate limit exceeded. Implement exponential backoff and retry logic.

**529 Provider Overloaded**: The provider is temporarily overloaded. Enable `allow_fallbacks: true` to automatically use backup providers.

## Limitations

- **No Streaming**: Unlike chat completions, embeddings are returned as complete responses. Streaming is not supported.
- **Token Limits**: Each model has a maximum input length. Texts exceeding this limit will be truncated or rejected.
- **Deterministic Output**: Embeddings for the same input text will always be identical (no temperature or randomness).
- **Language Support**: Some models are optimized for specific languages. Check model documentation for language capabilities.

## Related Resources

- [Models Page](https://openrouter.ai/models?fmt=cards&output_modalities=embeddings) - Browse all available embedding models
- [Provider Routing](/docs/features/provider-routing) - Control which providers serve your requests
- [Authentication](/docs/api/authentication) - Learn about API key authentication
- [Errors](/docs/api/errors) - Detailed error codes and handling


---
title: Responses API Beta
subtitle: OpenAI-compatible Responses API (Beta)
headline: OpenRouter Responses API Beta
canonical-url: 'https://openrouter.ai/docs/api/reference/responses/overview'
'og:site_name': OpenRouter Documentation
'og:title': OpenRouter Responses API Beta - OpenAI-Compatible Documentation
'og:description': >-
  Beta version of OpenRouter's OpenAI-compatible Responses API. Stateless
  transformation layer with support for reasoning, tool calling, and web search.
'og:image':
  type: url
  value: >-
    https://openrouter.ai/dynamic-og?title=Responses%20API%20Beta&description=OpenAI-compatible%20stateless%20API
'og:image:width': 1200
'og:image:height': 630
'twitter:card': summary_large_image
'twitter:site': '@OpenRouterAI'
noindex: false
nofollow: false
---

<Warning title="Beta API">
  This API is in **beta stage** and may have breaking changes. Use with caution in production environments.
</Warning>

<Info title="Stateless Only">
  This API is **stateless** - each request is independent and no conversation state is persisted between requests. You must include the full conversation history in each request.
</Info>

OpenRouter's Responses API Beta provides OpenAI-compatible access to multiple AI models through a unified interface, designed to be a drop-in replacement for OpenAI's Responses API. This stateless API offers enhanced capabilities including reasoning, tool calling, and web search integration, with each request being independent and no server-side state persisted.

## Base URL

```
https://openrouter.ai/api/v1/responses
```

## Authentication

All requests require authentication using your OpenRouter API key:

<CodeGroup>

```typescript title="TypeScript"
const response = await fetch('https://openrouter.ai/api/v1/responses', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer YOUR_OPENROUTER_API_KEY',
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    model: 'openai/o4-mini',
    input: 'Hello, world!',
  }),
});
```

```python title="Python"
import requests

response = requests.post(
    'https://openrouter.ai/api/v1/responses',
    headers={
        'Authorization': 'Bearer YOUR_OPENROUTER_API_KEY',
        'Content-Type': 'application/json',
    },
    json={
        'model': 'openai/o4-mini',
        'input': 'Hello, world!',
    }
)
```

```bash title="cURL"
curl -X POST https://openrouter.ai/api/v1/responses \
  -H "Authorization: Bearer YOUR_OPENROUTER_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "openai/o4-mini",
    "input": "Hello, world!"
  }'
```

</CodeGroup>

## Core Features

### [Basic Usage](./basic-usage)
Learn the fundamentals of making requests with simple text input and handling responses.

### [Reasoning](./reasoning)
Access advanced reasoning capabilities with configurable effort levels and encrypted reasoning chains.

### [Tool Calling](./tool-calling)
Integrate function calling with support for parallel execution and complex tool interactions.

### [Web Search](./web-search)
Enable web search capabilities with real-time information retrieval and citation annotations.


## Error Handling

The API returns structured error responses:

```json
{
  "error": {
    "code": "invalid_prompt",
    "message": "Missing required parameter: 'model'."
  },
  "metadata": null
}
```

For comprehensive error handling guidance, see [Error Handling](./error-handling).

## Rate Limits

Standard OpenRouter rate limits apply. See [API Limits](/docs/api-reference/limits) for details.


---
title: Basic Usage
subtitle: Getting started with the Responses API Beta
headline: Responses API Beta Basic Usage | Simple Text Requests
canonical-url: 'https://openrouter.ai/docs/api/reference/responses/basic-usage'
'og:site_name': OpenRouter Documentation
'og:title': Responses API Beta Basic Usage - Simple Text Requests
'og:description': >-
  Learn the basics of OpenRouter's Responses API Beta with simple text input
  examples and response handling.
'og:image':
  type: url
  value: >-
    https://openrouter.ai/dynamic-og?title=Responses%20API%20Basic%20Usage&description=Simple%20text%20requests%20and%20responses
'og:image:width': 1200
'og:image:height': 630
'twitter:card': summary_large_image
'twitter:site': '@OpenRouterAI'
noindex: false
nofollow: false
---

<Warning title="Beta API">
  This API is in **beta stage** and may have breaking changes.
</Warning>

The Responses API Beta supports both simple string input and structured message arrays, making it easy to get started with basic text generation.

## Simple String Input

The simplest way to use the API is with a string input:

<CodeGroup>

```typescript title="TypeScript"
const response = await fetch('https://openrouter.ai/api/v1/responses', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer YOUR_OPENROUTER_API_KEY',
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    model: 'openai/o4-mini',
    input: 'What is the meaning of life?',
    max_output_tokens: 9000,
  }),
});

const result = await response.json();
console.log(result);
```

```python title="Python"
import requests

response = requests.post(
    'https://openrouter.ai/api/v1/responses',
    headers={
        'Authorization': 'Bearer YOUR_OPENROUTER_API_KEY',
        'Content-Type': 'application/json',
    },
    json={
        'model': 'openai/o4-mini',
        'input': 'What is the meaning of life?',
        'max_output_tokens': 9000,
    }
)

result = response.json()
print(result)
```

```bash title="cURL"
curl -X POST https://openrouter.ai/api/v1/responses \
  -H "Authorization: Bearer YOUR_OPENROUTER_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "openai/o4-mini",
    "input": "What is the meaning of life?",
    "max_output_tokens": 9000
  }'
```

</CodeGroup>

## Structured Message Input

For more complex conversations, use the message array format:

<CodeGroup>

```typescript title="TypeScript"
const response = await fetch('https://openrouter.ai/api/v1/responses', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer YOUR_OPENROUTER_API_KEY',
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    model: 'openai/o4-mini',
    input: [
      {
        type: 'message',
        role: 'user',
        content: [
          {
            type: 'input_text',
            text: 'Tell me a joke about programming',
          },
        ],
      },
    ],
    max_output_tokens: 9000,
  }),
});

const result = await response.json();
```

```python title="Python"
import requests

response = requests.post(
    'https://openrouter.ai/api/v1/responses',
    headers={
        'Authorization': 'Bearer YOUR_OPENROUTER_API_KEY',
        'Content-Type': 'application/json',
    },
    json={
        'model': 'openai/o4-mini',
        'input': [
            {
                'type': 'message',
                'role': 'user',
                'content': [
                    {
                        'type': 'input_text',
                        'text': 'Tell me a joke about programming',
                    },
                ],
            },
        ],
        'max_output_tokens': 9000,
    }
)

result = response.json()
```

```bash title="cURL"
curl -X POST https://openrouter.ai/api/v1/responses \
  -H "Authorization: Bearer YOUR_OPENROUTER_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "openai/o4-mini",
    "input": [
      {
        "type": "message",
        "role": "user",
        "content": [
          {
            "type": "input_text",
            "text": "Tell me a joke about programming"
          }
        ]
      }
    ],
    "max_output_tokens": 9000
  }'
```

</CodeGroup>

## Response Format

The API returns a structured response with the generated content:

```json
{
  "id": "resp_1234567890",
  "object": "response",
  "created_at": 1234567890,
  "model": "openai/o4-mini",
  "output": [
    {
      "type": "message",
      "id": "msg_abc123",
      "status": "completed",
      "role": "assistant",
      "content": [
        {
          "type": "output_text",
          "text": "The meaning of life is a philosophical question that has been pondered for centuries...",
          "annotations": []
        }
      ]
    }
  ],
  "usage": {
    "input_tokens": 12,
    "output_tokens": 45,
    "total_tokens": 57
  },
  "status": "completed"
}
```

## Streaming Responses

Enable streaming for real-time response generation:

<CodeGroup>

```typescript title="TypeScript"
const response = await fetch('https://openrouter.ai/api/v1/responses', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer YOUR_OPENROUTER_API_KEY',
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    model: 'openai/o4-mini',
    input: 'Write a short story about AI',
    stream: true,
    max_output_tokens: 9000,
  }),
});

const reader = response.body?.getReader();
const decoder = new TextDecoder();

while (true) {
  const { done, value } = await reader.read();
  if (done) break;

  const chunk = decoder.decode(value);
  const lines = chunk.split('\n');

  for (const line of lines) {
    if (line.startsWith('data: ')) {
      const data = line.slice(6);
      if (data === '[DONE]') return;

      try {
        const parsed = JSON.parse(data);
        console.log(parsed);
      } catch (e) {
        // Skip invalid JSON
      }
    }
  }
}
```

```python title="Python"
import requests
import json

response = requests.post(
    'https://openrouter.ai/api/v1/responses',
    headers={
        'Authorization': 'Bearer YOUR_OPENROUTER_API_KEY',
        'Content-Type': 'application/json',
    },
    json={
        'model': 'openai/o4-mini',
        'input': 'Write a short story about AI',
        'stream': True,
        'max_output_tokens': 9000,
    },
    stream=True
)

for line in response.iter_lines():
    if line:
        line_str = line.decode('utf-8')
        if line_str.startswith('data: '):
            data = line_str[6:]
            if data == '[DONE]':
                break
            try:
                parsed = json.loads(data)
                print(parsed)
            except json.JSONDecodeError:
                continue
```

</CodeGroup>

### Example Streaming Output

The streaming response returns Server-Sent Events (SSE) chunks:

```
data: {"type":"response.created","response":{"id":"resp_1234567890","object":"response","status":"in_progress"}}

data: {"type":"response.output_item.added","response_id":"resp_1234567890","output_index":0,"item":{"type":"message","id":"msg_abc123","role":"assistant","status":"in_progress","content":[]}}

data: {"type":"response.content_part.added","response_id":"resp_1234567890","output_index":0,"content_index":0,"part":{"type":"output_text","text":""}}

data: {"type":"response.content_part.delta","response_id":"resp_1234567890","output_index":0,"content_index":0,"delta":"Once"}

data: {"type":"response.content_part.delta","response_id":"resp_1234567890","output_index":0,"content_index":0,"delta":" upon"}

data: {"type":"response.content_part.delta","response_id":"resp_1234567890","output_index":0,"content_index":0,"delta":" a"}

data: {"type":"response.content_part.delta","response_id":"resp_1234567890","output_index":0,"content_index":0,"delta":" time"}

data: {"type":"response.output_item.done","response_id":"resp_1234567890","output_index":0,"item":{"type":"message","id":"msg_abc123","role":"assistant","status":"completed","content":[{"type":"output_text","text":"Once upon a time, in a world where artificial intelligence had become as common as smartphones..."}]}}

data: {"type":"response.done","response":{"id":"resp_1234567890","object":"response","status":"completed","usage":{"input_tokens":12,"output_tokens":45,"total_tokens":57}}}

data: [DONE]
```

## Common Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `model` | string | **Required.** Model to use (e.g., `openai/o4-mini`) |
| `input` | string or array | **Required.** Text or message array |
| `stream` | boolean | Enable streaming responses (default: false) |
| `max_output_tokens` | integer | Maximum tokens to generate |
| `temperature` | number | Sampling temperature (0-2) |
| `top_p` | number | Nucleus sampling parameter (0-1) |

## Error Handling

Handle common errors gracefully:

<CodeGroup>

```typescript title="TypeScript"
try {
  const response = await fetch('https://openrouter.ai/api/v1/responses', {
    method: 'POST',
    headers: {
      'Authorization': 'Bearer YOUR_OPENROUTER_API_KEY',
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      model: 'openai/o4-mini',
      input: 'Hello, world!',
    }),
  });

  if (!response.ok) {
    const error = await response.json();
    console.error('API Error:', error.error.message);
    return;
  }

  const result = await response.json();
  console.log(result);
} catch (error) {
  console.error('Network Error:', error);
}
```

```python title="Python"
import requests

try:
    response = requests.post(
        'https://openrouter.ai/api/v1/responses',
        headers={
            'Authorization': 'Bearer YOUR_OPENROUTER_API_KEY',
            'Content-Type': 'application/json',
        },
        json={
            'model': 'openai/o4-mini',
            'input': 'Hello, world!',
        }
    )

    if response.status_code != 200:
        error = response.json()
        print(f"API Error: {error['error']['message']}")
    else:
        result = response.json()
        print(result)

except requests.RequestException as e:
    print(f"Network Error: {e}")
```

</CodeGroup>

## Multiple Turn Conversations

Since the Responses API Beta is stateless, you must include the full conversation history in each request to maintain context:

<CodeGroup>

```typescript title="TypeScript"
// First request
const firstResponse = await fetch('https://openrouter.ai/api/beta/responses', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer YOUR_OPENROUTER_API_KEY',
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    model: 'openai/o4-mini',
    input: [
      {
        type: 'message',
        role: 'user',
        content: [
          {
            type: 'input_text',
            text: 'What is the capital of France?',
          },
        ],
      },
    ],
    max_output_tokens: 9000,
  }),
});

const firstResult = await firstResponse.json();

// Second request - include previous conversation
const secondResponse = await fetch('https://openrouter.ai/api/beta/responses', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer YOUR_OPENROUTER_API_KEY',
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    model: 'openai/o4-mini',
    input: [
      {
        type: 'message',
        role: 'user',
        content: [
          {
            type: 'input_text',
            text: 'What is the capital of France?',
          },
        ],
      },
      {
        type: 'message',
        role: 'assistant',
        id: 'msg_abc123',
        status: 'completed',
        content: [
          {
            type: 'output_text',
            text: 'The capital of France is Paris.',
            annotations: []
          }
        ]
      },
      {
        type: 'message',
        role: 'user',
        content: [
          {
            type: 'input_text',
            text: 'What is the population of that city?',
          },
        ],
      },
    ],
    max_output_tokens: 9000,
  }),
});

const secondResult = await secondResponse.json();
```

```python title="Python"
import requests

# First request
first_response = requests.post(
    'https://openrouter.ai/api/v1/responses',
    headers={
        'Authorization': 'Bearer YOUR_OPENROUTER_API_KEY',
        'Content-Type': 'application/json',
    },
    json={
        'model': 'openai/o4-mini',
        'input': [
            {
                'type': 'message',
                'role': 'user',
                'content': [
                    {
                        'type': 'input_text',
                        'text': 'What is the capital of France?',
                    },
                ],
            },
        ],
        'max_output_tokens': 9000,
    }
)

first_result = first_response.json()

# Second request - include previous conversation
second_response = requests.post(
    'https://openrouter.ai/api/v1/responses',
    headers={
        'Authorization': 'Bearer YOUR_OPENROUTER_API_KEY',
        'Content-Type': 'application/json',
    },
    json={
        'model': 'openai/o4-mini',
        'input': [
            {
                'type': 'message',
                'role': 'user',
                'content': [
                    {
                        'type': 'input_text',
                        'text': 'What is the capital of France?',
                    },
                ],
            },
            {
                'type': 'message',
                'role': 'assistant',
                'id': 'msg_abc123',
                'status': 'completed',
                'content': [
                    {
                        'type': 'output_text',
                        'text': 'The capital of France is Paris.',
                        'annotations': []
                    }
                ]
            },
            {
                'type': 'message',
                'role': 'user',
                'content': [
                    {
                        'type': 'input_text',
                        'text': 'What is the population of that city?',
                    },
                ],
            },
        ],
        'max_output_tokens': 9000,
    }
)

second_result = second_response.json()
```

</CodeGroup>

<Info title="Required Fields">
  The `id` and `status` fields are required for any `assistant` role messages included in the conversation history.
</Info>

<Info title="Conversation History">
  Always include the complete conversation history in each request. The API does not store previous messages, so context must be maintained client-side.
</Info>

## Next Steps

- Learn about [Reasoning](./reasoning) capabilities
- Explore [Tool Calling](./tool-calling) functionality
- Try [Web Search](./web-search) integration


---
title: Reasoning
subtitle: Advanced reasoning capabilities with the Responses API Beta
headline: Responses API Beta Reasoning | Advanced AI Reasoning Capabilities
canonical-url: 'https://openrouter.ai/docs/api/reference/responses/reasoning'
'og:site_name': OpenRouter Documentation
'og:title': Responses API Beta Reasoning - Advanced AI Reasoning
'og:description': >-
  Access advanced reasoning capabilities with configurable effort levels and
  encrypted reasoning chains using OpenRouter's Responses API Beta.
'og:image':
  type: url
  value: >-
    https://openrouter.ai/dynamic-og?title=Responses%20API%20Reasoning&description=Advanced%20AI%20reasoning%20capabilities
'og:image:width': 1200
'og:image:height': 630
'twitter:card': summary_large_image
'twitter:site': '@OpenRouterAI'
noindex: false
nofollow: false
---

<Warning title="Beta API">
  This API is in **beta stage** and may have breaking changes.
</Warning>

The Responses API Beta supports advanced reasoning capabilities, allowing models to show their internal reasoning process with configurable effort levels.

## Reasoning Configuration

Configure reasoning behavior using the `reasoning` parameter:

<CodeGroup>

```typescript title="TypeScript"
const response = await fetch('https://openrouter.ai/api/v1/responses', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer YOUR_OPENROUTER_API_KEY',
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    model: 'openai/o4-mini',
    input: 'What is the meaning of life?',
    reasoning: {
      effort: 'high'
    },
    max_output_tokens: 9000,
  }),
});

const result = await response.json();
console.log(result);
```

```python title="Python"
import requests

response = requests.post(
    'https://openrouter.ai/api/v1/responses',
    headers={
        'Authorization': 'Bearer YOUR_OPENROUTER_API_KEY',
        'Content-Type': 'application/json',
    },
    json={
        'model': 'openai/o4-mini',
        'input': 'What is the meaning of life?',
        'reasoning': {
            'effort': 'high'
        },
        'max_output_tokens': 9000,
    }
)

result = response.json()
print(result)
```

```bash title="cURL"
curl -X POST https://openrouter.ai/api/v1/responses \
  -H "Authorization: Bearer YOUR_OPENROUTER_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "openai/o4-mini",
    "input": "What is the meaning of life?",
    "reasoning": {
      "effort": "high"
    },
    "max_output_tokens": 9000
  }'
```

</CodeGroup>

## Reasoning Effort Levels

The `effort` parameter controls how much computational effort the model puts into reasoning:

| Effort Level | Description |
|--------------|-------------|
| `minimal` | Basic reasoning with minimal computational effort |
| `low` | Light reasoning for simple problems |
| `medium` | Balanced reasoning for moderate complexity |
| `high` | Deep reasoning for complex problems |

## Complex Reasoning Example

For complex mathematical or logical problems:

<CodeGroup>

```typescript title="TypeScript"
const response = await fetch('https://openrouter.ai/api/v1/responses', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer YOUR_OPENROUTER_API_KEY',
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    model: 'openai/o4-mini',
    input: [
      {
        type: 'message',
        role: 'user',
        content: [
          {
            type: 'input_text',
            text: 'Was 1995 30 years ago? Please show your reasoning.',
          },
        ],
      },
    ],
    reasoning: {
      effort: 'high'
    },
    max_output_tokens: 9000,
  }),
});

const result = await response.json();
console.log(result);
```

```python title="Python"
import requests

response = requests.post(
    'https://openrouter.ai/api/v1/responses',
    headers={
        'Authorization': 'Bearer YOUR_OPENROUTER_API_KEY',
        'Content-Type': 'application/json',
    },
    json={
        'model': 'openai/o4-mini',
        'input': [
            {
                'type': 'message',
                'role': 'user',
                'content': [
                    {
                        'type': 'input_text',
                        'text': 'Was 1995 30 years ago? Please show your reasoning.',
                    },
                ],
            },
        ],
        'reasoning': {
            'effort': 'high'
        },
        'max_output_tokens': 9000,
    }
)

result = response.json()
print(result)
```

</CodeGroup>

## Reasoning in Conversation Context

Include reasoning in multi-turn conversations:

<CodeGroup>

```typescript title="TypeScript"
const response = await fetch('https://openrouter.ai/api/v1/responses', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer YOUR_OPENROUTER_API_KEY',
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    model: 'openai/o4-mini',
    input: [
      {
        type: 'message',
        role: 'user',
        content: [
          {
            type: 'input_text',
            text: 'What is your favorite color?',
          },
        ],
      },
      {
        type: 'message',
        role: 'assistant',
        id: 'msg_abc123',
        status: 'completed',
        content: [
          {
            type: 'output_text',
            text: "I don't have a favorite color.",
            annotations: []
          }
        ]
      },
      {
        type: 'message',
        role: 'user',
        content: [
          {
            type: 'input_text',
            text: 'How many Earths can fit on Mars?',
          },
        ],
      },
    ],
    reasoning: {
      effort: 'high'
    },
    max_output_tokens: 9000,
  }),
});

const result = await response.json();
console.log(result);
```

```python title="Python"
import requests

response = requests.post(
    'https://openrouter.ai/api/v1/responses',
    headers={
        'Authorization': 'Bearer YOUR_OPENROUTER_API_KEY',
        'Content-Type': 'application/json',
    },
    json={
        'model': 'openai/o4-mini',
        'input': [
            {
                'type': 'message',
                'role': 'user',
                'content': [
                    {
                        'type': 'input_text',
                        'text': 'What is your favorite color?',
                    },
                ],
            },
            {
                'type': 'message',
                'role': 'assistant',
                'id': 'msg_abc123',
                'status': 'completed',
                'content': [
                    {
                        'type': 'output_text',
                        'text': "I don't have a favorite color.",
                        'annotations': []
                    }
                ]
            },
            {
                'type': 'message',
                'role': 'user',
                'content': [
                    {
                        'type': 'input_text',
                        'text': 'How many Earths can fit on Mars?',
                    },
                ],
            },
        ],
        'reasoning': {
            'effort': 'high'
        },
        'max_output_tokens': 9000,
    }
)

result = response.json()
print(result)
```

</CodeGroup>

## Streaming Reasoning

Enable streaming to see reasoning develop in real-time:

<CodeGroup>

```typescript title="TypeScript"
const response = await fetch('https://openrouter.ai/api/v1/responses', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer YOUR_OPENROUTER_API_KEY',
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    model: 'openai/o4-mini',
    input: 'Solve this step by step: If a train travels 60 mph for 2.5 hours, how far does it go?',
    reasoning: {
      effort: 'medium'
    },
    stream: true,
    max_output_tokens: 9000,
  }),
});

const reader = response.body?.getReader();
const decoder = new TextDecoder();

while (true) {
  const { done, value } = await reader.read();
  if (done) break;

  const chunk = decoder.decode(value);
  const lines = chunk.split('\n');

  for (const line of lines) {
    if (line.startsWith('data: ')) {
      const data = line.slice(6);
      if (data === '[DONE]') return;

      try {
        const parsed = JSON.parse(data);
        if (parsed.type === 'response.reasoning.delta') {
          console.log('Reasoning:', parsed.delta);
        }
      } catch (e) {
        // Skip invalid JSON
      }
    }
  }
}
```

```python title="Python"
import requests
import json

response = requests.post(
    'https://openrouter.ai/api/v1/responses',
    headers={
        'Authorization': 'Bearer YOUR_OPENROUTER_API_KEY',
        'Content-Type': 'application/json',
    },
    json={
        'model': 'openai/o4-mini',
        'input': 'Solve this step by step: If a train travels 60 mph for 2.5 hours, how far does it go?',
        'reasoning': {
            'effort': 'medium'
        },
        'stream': True,
        'max_output_tokens': 9000,
    },
    stream=True
)

for line in response.iter_lines():
    if line:
        line_str = line.decode('utf-8')
        if line_str.startswith('data: '):
            data = line_str[6:]
            if data == '[DONE]':
                break
            try:
                parsed = json.loads(data)
                if parsed.get('type') == 'response.reasoning.delta':
                    print(f"Reasoning: {parsed.get('delta', '')}")
            except json.JSONDecodeError:
                continue
```

</CodeGroup>

## Response with Reasoning

When reasoning is enabled, the response includes reasoning information:

```json
{
  "id": "resp_1234567890",
  "object": "response",
  "created_at": 1234567890,
  "model": "openai/o4-mini",
  "output": [
    {
      "type": "reasoning",
      "id": "rs_abc123",
      "encrypted_content": "gAAAAABotI9-FK1PbhZhaZk4yMrZw3XDI1AWFaKb9T0NQq7LndK6zaRB...",
      "summary": [
        "First, I need to determine the current year",
        "Then calculate the difference from 1995",
        "Finally, compare that to 30 years"
      ]
    },
    {
      "type": "message",
      "id": "msg_xyz789",
      "status": "completed",
      "role": "assistant",
      "content": [
        {
          "type": "output_text",
          "text": "Yes. In 2025, 1995 was 30 years ago. In fact, as of today (Aug 31, 2025), it's exactly 30 years since Aug 31, 1995.",
          "annotations": []
        }
      ]
    }
  ],
  "usage": {
    "input_tokens": 15,
    "output_tokens": 85,
    "output_tokens_details": {
      "reasoning_tokens": 45
    },
    "total_tokens": 100
  },
  "status": "completed"
}
```

## Best Practices

1. **Choose appropriate effort levels**: Use `high` for complex problems, `low` for simple tasks
2. **Consider token usage**: Reasoning increases token consumption
3. **Use streaming**: For long reasoning chains, streaming provides better user experience
4. **Include context**: Provide sufficient context for the model to reason effectively

## Next Steps

- Explore [Tool Calling](./tool-calling) with reasoning
- Learn about [Web Search](./web-search) integration
- Review [Basic Usage](./basic-usage) fundamentals


---
title: Tool Calling
subtitle: Function calling and tool integration with the Responses API Beta
headline: Responses API Beta Tool Calling | Function Calling Integration
canonical-url: 'https://openrouter.ai/docs/api/reference/responses/tool-calling'
'og:site_name': OpenRouter Documentation
'og:title': Responses API Beta Tool Calling - Function Calling Integration
'og:description': >-
  Integrate function calling with support for parallel execution and complex
  tool interactions using OpenRouter's Responses API Beta.
'og:image':
  type: url
  value: >-
    https://openrouter.ai/dynamic-og?title=Responses%20API%20Tool%20Calling&description=Function%20calling%20integration
'og:image:width': 1200
'og:image:height': 630
'twitter:card': summary_large_image
'twitter:site': '@OpenRouterAI'
noindex: false
nofollow: false
---

<Warning title="Beta API">
  This API is in **beta stage** and may have breaking changes.
</Warning>

The Responses API Beta supports comprehensive tool calling capabilities, allowing models to call functions, execute tools in parallel, and handle complex multi-step workflows.

## Basic Tool Definition

Define tools using the OpenAI function calling format:

<CodeGroup>

```typescript title="TypeScript"
const weatherTool = {
  type: 'function' as const,
  name: 'get_weather',
  description: 'Get the current weather in a location',
  strict: null,
  parameters: {
    type: 'object',
    properties: {
      location: {
        type: 'string',
        description: 'The city and state, e.g. San Francisco, CA',
      },
      unit: {
        type: 'string',
        enum: ['celsius', 'fahrenheit'],
      },
    },
    required: ['location'],
  },
};

const response = await fetch('https://openrouter.ai/api/v1/responses', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer YOUR_OPENROUTER_API_KEY',
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    model: 'openai/o4-mini',
    input: [
      {
        type: 'message',
        role: 'user',
        content: [
          {
            type: 'input_text',
            text: 'What is the weather in San Francisco?',
          },
        ],
      },
    ],
    tools: [weatherTool],
    tool_choice: 'auto',
    max_output_tokens: 9000,
  }),
});

const result = await response.json();
console.log(result);
```

```python title="Python"
import requests

weather_tool = {
    'type': 'function',
    'name': 'get_weather',
    'description': 'Get the current weather in a location',
    'strict': None,
    'parameters': {
        'type': 'object',
        'properties': {
            'location': {
                'type': 'string',
                'description': 'The city and state, e.g. San Francisco, CA',
            },
            'unit': {
                'type': 'string',
                'enum': ['celsius', 'fahrenheit'],
            },
        },
        'required': ['location'],
    },
}

response = requests.post(
    'https://openrouter.ai/api/v1/responses',
    headers={
        'Authorization': 'Bearer YOUR_OPENROUTER_API_KEY',
        'Content-Type': 'application/json',
    },
    json={
        'model': 'openai/o4-mini',
        'input': [
            {
                'type': 'message',
                'role': 'user',
                'content': [
                    {
                        'type': 'input_text',
                        'text': 'What is the weather in San Francisco?',
                    },
                ],
            },
        ],
        'tools': [weather_tool],
        'tool_choice': 'auto',
        'max_output_tokens': 9000,
    }
)

result = response.json()
print(result)
```

```bash title="cURL"
curl -X POST https://openrouter.ai/api/v1/responses \
  -H "Authorization: Bearer YOUR_OPENROUTER_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "openai/o4-mini",
    "input": [
      {
        "type": "message",
        "role": "user",
        "content": [
          {
            "type": "input_text",
            "text": "What is the weather in San Francisco?"
          }
        ]
      }
    ],
    "tools": [
      {
        "type": "function",
        "name": "get_weather",
        "description": "Get the current weather in a location",
        "strict": null,
        "parameters": {
          "type": "object",
          "properties": {
            "location": {
              "type": "string",
              "description": "The city and state, e.g. San Francisco, CA"
            },
            "unit": {
              "type": "string",
              "enum": ["celsius", "fahrenheit"]
            }
          },
          "required": ["location"]
        }
      }
    ],
    "tool_choice": "auto",
    "max_output_tokens": 9000
  }'
```

</CodeGroup>

## Tool Choice Options

Control when and how tools are called:

| Tool Choice | Description |
|-------------|-------------|
| `auto` | Model decides whether to call tools |
| `none` | Model will not call any tools |
| `{type: 'function', name: 'tool_name'}` | Force specific tool call |

### Force Specific Tool

<CodeGroup>

```typescript title="TypeScript"
const response = await fetch('https://openrouter.ai/api/v1/responses', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer YOUR_OPENROUTER_API_KEY',
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    model: 'openai/o4-mini',
    input: [
      {
        type: 'message',
        role: 'user',
        content: [
          {
            type: 'input_text',
            text: 'Hello, how are you?',
          },
        ],
      },
    ],
    tools: [weatherTool],
    tool_choice: { type: 'function', name: 'get_weather' },
    max_output_tokens: 9000,
  }),
});
```

```python title="Python"
import requests

response = requests.post(
    'https://openrouter.ai/api/v1/responses',
    headers={
        'Authorization': 'Bearer YOUR_OPENROUTER_API_KEY',
        'Content-Type': 'application/json',
    },
    json={
        'model': 'openai/o4-mini',
        'input': [
            {
                'type': 'message',
                'role': 'user',
                'content': [
                    {
                        'type': 'input_text',
                        'text': 'Hello, how are you?',
                    },
                ],
            },
        ],
        'tools': [weather_tool],
        'tool_choice': {'type': 'function', 'name': 'get_weather'},
        'max_output_tokens': 9000,
    }
)
```

</CodeGroup>

### Disable Tool Calling

<CodeGroup>

```typescript title="TypeScript"
const response = await fetch('https://openrouter.ai/api/v1/responses', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer YOUR_OPENROUTER_API_KEY',
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    model: 'openai/o4-mini',
    input: [
      {
        type: 'message',
        role: 'user',
        content: [
          {
            type: 'input_text',
            text: 'What is the weather in Paris?',
          },
        ],
      },
    ],
    tools: [weatherTool],
    tool_choice: 'none',
    max_output_tokens: 9000,
  }),
});
```

```python title="Python"
import requests

response = requests.post(
    'https://openrouter.ai/api/v1/responses',
    headers={
        'Authorization': 'Bearer YOUR_OPENROUTER_API_KEY',
        'Content-Type': 'application/json',
    },
    json={
        'model': 'openai/o4-mini',
        'input': [
            {
                'type': 'message',
                'role': 'user',
                'content': [
                    {
                        'type': 'input_text',
                        'text': 'What is the weather in Paris?',
                    },
                ],
            },
        ],
        'tools': [weather_tool],
        'tool_choice': 'none',
        'max_output_tokens': 9000,
    }
)
```

</CodeGroup>

## Multiple Tools

Define multiple tools for complex workflows:

<CodeGroup>

```typescript title="TypeScript"
const calculatorTool = {
  type: 'function' as const,
  name: 'calculate',
  description: 'Perform mathematical calculations',
  strict: null,
  parameters: {
    type: 'object',
    properties: {
      expression: {
        type: 'string',
        description: 'The mathematical expression to evaluate',
      },
    },
    required: ['expression'],
  },
};

const response = await fetch('https://openrouter.ai/api/v1/responses', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer YOUR_OPENROUTER_API_KEY',
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    model: 'openai/o4-mini',
    input: [
      {
        type: 'message',
        role: 'user',
        content: [
          {
            type: 'input_text',
            text: 'What is 25 * 4?',
          },
        ],
      },
    ],
    tools: [weatherTool, calculatorTool],
    tool_choice: 'auto',
    max_output_tokens: 9000,
  }),
});
```

```python title="Python"
calculator_tool = {
    'type': 'function',
    'name': 'calculate',
    'description': 'Perform mathematical calculations',
    'strict': None,
    'parameters': {
        'type': 'object',
        'properties': {
            'expression': {
                'type': 'string',
                'description': 'The mathematical expression to evaluate',
            },
        },
        'required': ['expression'],
    },
}

response = requests.post(
    'https://openrouter.ai/api/v1/responses',
    headers={
        'Authorization': 'Bearer YOUR_OPENROUTER_API_KEY',
        'Content-Type': 'application/json',
    },
    json={
        'model': 'openai/o4-mini',
        'input': [
            {
                'type': 'message',
                'role': 'user',
                'content': [
                    {
                        'type': 'input_text',
                        'text': 'What is 25 * 4?',
                    },
                ],
            },
        ],
        'tools': [weather_tool, calculator_tool],
        'tool_choice': 'auto',
        'max_output_tokens': 9000,
    }
)
```

</CodeGroup>

## Parallel Tool Calls

The API supports parallel execution of multiple tools:

<CodeGroup>

```typescript title="TypeScript"
const response = await fetch('https://openrouter.ai/api/v1/responses', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer YOUR_OPENROUTER_API_KEY',
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    model: 'openai/o4-mini',
    input: [
      {
        type: 'message',
        role: 'user',
        content: [
          {
            type: 'input_text',
            text: 'Calculate 10*5 and also tell me the weather in Miami',
          },
        ],
      },
    ],
    tools: [weatherTool, calculatorTool],
    tool_choice: 'auto',
    max_output_tokens: 9000,
  }),
});

const result = await response.json();
console.log(result);
```

```python title="Python"
import requests

response = requests.post(
    'https://openrouter.ai/api/v1/responses',
    headers={
        'Authorization': 'Bearer YOUR_OPENROUTER_API_KEY',
        'Content-Type': 'application/json',
    },
    json={
        'model': 'openai/o4-mini',
        'input': [
            {
                'type': 'message',
                'role': 'user',
                'content': [
                    {
                        'type': 'input_text',
                        'text': 'Calculate 10*5 and also tell me the weather in Miami',
                    },
                ],
            },
        ],
        'tools': [weather_tool, calculator_tool],
        'tool_choice': 'auto',
        'max_output_tokens': 9000,
    }
)

result = response.json()
print(result)
```

</CodeGroup>

## Tool Call Response

When tools are called, the response includes function call information:

```json
{
  "id": "resp_1234567890",
  "object": "response",
  "created_at": 1234567890,
  "model": "openai/o4-mini",
  "output": [
    {
      "type": "function_call",
      "id": "fc_abc123",
      "call_id": "call_xyz789",
      "name": "get_weather",
      "arguments": "{\"location\":\"San Francisco, CA\"}"
    }
  ],
  "usage": {
    "input_tokens": 45,
    "output_tokens": 25,
    "total_tokens": 70
  },
  "status": "completed"
}
```

## Tool Responses in Conversation

Include tool responses in follow-up requests:

<CodeGroup>

```typescript title="TypeScript"
const response = await fetch('https://openrouter.ai/api/v1/responses', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer YOUR_OPENROUTER_API_KEY',
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    model: 'openai/o4-mini',
    input: [
      {
        type: 'message',
        role: 'user',
        content: [
          {
            type: 'input_text',
            text: 'What is the weather in Boston?',
          },
        ],
      },
      {
        type: 'function_call',
        id: 'fc_1',
        call_id: 'call_123',
        name: 'get_weather',
        arguments: JSON.stringify({ location: 'Boston, MA' }),
      },
      {
        type: 'function_call_output',
        id: 'fc_output_1',
        call_id: 'call_123',
        output: JSON.stringify({ temperature: '72°F', condition: 'Sunny' }),
      },
      {
        type: 'message',
        role: 'assistant',
        id: 'msg_abc123',
        status: 'completed',
        content: [
          {
            type: 'output_text',
            text: 'The weather in Boston is currently 72°F and sunny. This looks like perfect weather for a picnic!',
            annotations: []
          }
        ]
      },
      {
        type: 'message',
        role: 'user',
        content: [
          {
            type: 'input_text',
            text: 'Is that good weather for a picnic?',
          },
        ],
      },
    ],
    max_output_tokens: 9000,
  }),
});
```

```python title="Python"
import requests

response = requests.post(
    'https://openrouter.ai/api/v1/responses',
    headers={
        'Authorization': 'Bearer YOUR_OPENROUTER_API_KEY',
        'Content-Type': 'application/json',
    },
    json={
        'model': 'openai/o4-mini',
        'input': [
            {
                'type': 'message',
                'role': 'user',
                'content': [
                    {
                        'type': 'input_text',
                        'text': 'What is the weather in Boston?',
                    },
                ],
            },
            {
                'type': 'function_call',
                'id': 'fc_1',
                'call_id': 'call_123',
                'name': 'get_weather',
                'arguments': '{"location": "Boston, MA"}',
            },
            {
                'type': 'function_call_output',
                'id': 'fc_output_1',
                'call_id': 'call_123',
                'output': '{"temperature": "72°F", "condition": "Sunny"}',
            },
            {
                'type': 'message',
                'role': 'assistant',
                'id': 'msg_abc123',
                'status': 'completed',
                'content': [
                    {
                        'type': 'output_text',
                        'text': 'The weather in Boston is currently 72°F and sunny. This looks like perfect weather for a picnic!',
                        'annotations': []
                    }
                ]
            },
            {
                'type': 'message',
                'role': 'user',
                'content': [
                    {
                        'type': 'input_text',
                        'text': 'Is that good weather for a picnic?',
                    },
                ],
            },
        ],
        'max_output_tokens': 9000,
    }
)
```

</CodeGroup>

<Info title="Required Field">
  The `id` field is required for `function_call_output` objects when including tool responses in conversation history.
</Info>

## Streaming Tool Calls

Monitor tool calls in real-time with streaming:

<CodeGroup>

```typescript title="TypeScript"
const response = await fetch('https://openrouter.ai/api/v1/responses', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer YOUR_OPENROUTER_API_KEY',
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    model: 'openai/o4-mini',
    input: [
      {
        type: 'message',
        role: 'user',
        content: [
          {
            type: 'input_text',
            text: 'What is the weather like in Tokyo, Japan? Please check the weather.',
          },
        ],
      },
    ],
    tools: [weatherTool],
    tool_choice: 'auto',
    stream: true,
    max_output_tokens: 9000,
  }),
});

const reader = response.body?.getReader();
const decoder = new TextDecoder();

while (true) {
  const { done, value } = await reader.read();
  if (done) break;

  const chunk = decoder.decode(value);
  const lines = chunk.split('\n');

  for (const line of lines) {
    if (line.startsWith('data: ')) {
      const data = line.slice(6);
      if (data === '[DONE]') return;

      try {
        const parsed = JSON.parse(data);
        if (parsed.type === 'response.output_item.added' &&
            parsed.item?.type === 'function_call') {
          console.log('Function call:', parsed.item.name);
        }
        if (parsed.type === 'response.function_call_arguments.done') {
          console.log('Arguments:', parsed.arguments);
        }
      } catch (e) {
        // Skip invalid JSON
      }
    }
  }
}
```

```python title="Python"
import requests
import json

response = requests.post(
    'https://openrouter.ai/api/v1/responses',
    headers={
        'Authorization': 'Bearer YOUR_OPENROUTER_API_KEY',
        'Content-Type': 'application/json',
    },
    json={
        'model': 'openai/o4-mini',
        'input': [
            {
                'type': 'message',
                'role': 'user',
                'content': [
                    {
                        'type': 'input_text',
                        'text': 'What is the weather like in Tokyo, Japan? Please check the weather.',
                    },
                ],
            },
        ],
        'tools': [weather_tool],
        'tool_choice': 'auto',
        'stream': True,
        'max_output_tokens': 9000,
    },
    stream=True
)

for line in response.iter_lines():
    if line:
        line_str = line.decode('utf-8')
        if line_str.startswith('data: '):
            data = line_str[6:]
            if data == '[DONE]':
                break
            try:
                parsed = json.loads(data)
                if (parsed.get('type') == 'response.output_item.added' and
                    parsed.get('item', {}).get('type') == 'function_call'):
                    print(f"Function call: {parsed['item']['name']}")
                if parsed.get('type') == 'response.function_call_arguments.done':
                    print(f"Arguments: {parsed.get('arguments', '')}")
            except json.JSONDecodeError:
                continue
```

</CodeGroup>

## Tool Validation

Ensure tool calls have proper structure:

```json
{
  "type": "function_call",
  "id": "fc_abc123",
  "call_id": "call_xyz789",
  "name": "get_weather",
  "arguments": "{\"location\":\"Seattle, WA\"}"
}
```

Required fields:
- `type`: Always "function_call"
- `id`: Unique identifier for the function call object
- `name`: Function name matching tool definition
- `arguments`: Valid JSON string with function parameters
- `call_id`: Unique identifier for the call

## Best Practices

1. **Clear descriptions**: Provide detailed function descriptions and parameter explanations
2. **Proper schemas**: Use valid JSON Schema for parameters
3. **Error handling**: Handle cases where tools might not be called
4. **Parallel execution**: Design tools to work independently when possible
5. **Conversation flow**: Include tool responses in follow-up requests for context

## Next Steps

- Learn about [Web Search](./web-search) integration
- Explore [Reasoning](./reasoning) with tools
- Review [Basic Usage](./basic-usage) fundamentals


---
title: Web Search
subtitle: Real-time web search integration with the Responses API Beta
headline: Responses API Beta Web Search | Real-time Information Retrieval
canonical-url: 'https://openrouter.ai/docs/api/reference/responses/web-search'
'og:site_name': OpenRouter Documentation
'og:title': Responses API Beta Web Search - Real-time Information Retrieval
'og:description': >-
  Enable web search capabilities with real-time information retrieval and
  citation annotations using OpenRouter's Responses API Beta.
'og:image':
  type: url
  value: >-
    https://openrouter.ai/dynamic-og?title=Responses%20API%20Web%20Search&description=Real-time%20information%20retrieval
'og:image:width': 1200
'og:image:height': 630
'twitter:card': summary_large_image
'twitter:site': '@OpenRouterAI'
noindex: false
nofollow: false
---

<Warning title="Beta API">
  This API is in **beta stage** and may have breaking changes.
</Warning>

The Responses API Beta supports web search integration, allowing models to access real-time information from the internet and provide responses with proper citations and annotations.

## Web Search Plugin

Enable web search using the `plugins` parameter:

<CodeGroup>

```typescript title="TypeScript"
const response = await fetch('https://openrouter.ai/api/v1/responses', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer YOUR_OPENROUTER_API_KEY',
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    model: 'openai/o4-mini',
    input: 'What is OpenRouter?',
    plugins: [{ id: 'web', max_results: 3 }],
    max_output_tokens: 9000,
  }),
});

const result = await response.json();
console.log(result);
```

```python title="Python"
import requests

response = requests.post(
    'https://openrouter.ai/api/v1/responses',
    headers={
        'Authorization': 'Bearer YOUR_OPENROUTER_API_KEY',
        'Content-Type': 'application/json',
    },
    json={
        'model': 'openai/o4-mini',
        'input': 'What is OpenRouter?',
        'plugins': [{'id': 'web', 'max_results': 3}],
        'max_output_tokens': 9000,
    }
)

result = response.json()
print(result)
```

```bash title="cURL"
curl -X POST https://openrouter.ai/api/v1/responses \
  -H "Authorization: Bearer YOUR_OPENROUTER_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "openai/o4-mini",
    "input": "What is OpenRouter?",
    "plugins": [{"id": "web", "max_results": 3}],
    "max_output_tokens": 9000
  }'
```

</CodeGroup>

## Plugin Configuration

Configure web search behavior:

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | string | **Required.** Must be "web" |
| `max_results` | integer | Maximum search results to retrieve (1-10) |

## Structured Message with Web Search

Use structured messages for more complex queries:

<CodeGroup>

```typescript title="TypeScript"
const response = await fetch('https://openrouter.ai/api/v1/responses', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer YOUR_OPENROUTER_API_KEY',
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    model: 'openai/o4-mini',
    input: [
      {
        type: 'message',
        role: 'user',
        content: [
          {
            type: 'input_text',
            text: 'What was a positive news story from today?',
          },
        ],
      },
    ],
    plugins: [{ id: 'web', max_results: 2 }],
    max_output_tokens: 9000,
  }),
});

const result = await response.json();
console.log(result);
```

```python title="Python"
import requests

response = requests.post(
    'https://openrouter.ai/api/v1/responses',
    headers={
        'Authorization': 'Bearer YOUR_OPENROUTER_API_KEY',
        'Content-Type': 'application/json',
    },
    json={
        'model': 'openai/o4-mini',
        'input': [
            {
                'type': 'message',
                'role': 'user',
                'content': [
                    {
                        'type': 'input_text',
                        'text': 'What was a positive news story from today?',
                    },
                ],
            },
        ],
        'plugins': [{'id': 'web', 'max_results': 2}],
        'max_output_tokens': 9000,
    }
)

result = response.json()
print(result)
```

</CodeGroup>

## Online Model Variants

Some models have built-in web search capabilities using the `:online` variant:

<CodeGroup>

```typescript title="TypeScript"
const response = await fetch('https://openrouter.ai/api/v1/responses', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer YOUR_OPENROUTER_API_KEY',
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    model: 'openai/o4-mini:online',
    input: 'What was a positive news story from today?',
    max_output_tokens: 9000,
  }),
});

const result = await response.json();
console.log(result);
```

```python title="Python"
import requests

response = requests.post(
    'https://openrouter.ai/api/v1/responses',
    headers={
        'Authorization': 'Bearer YOUR_OPENROUTER_API_KEY',
        'Content-Type': 'application/json',
    },
    json={
        'model': 'openai/o4-mini:online',
        'input': 'What was a positive news story from today?',
        'max_output_tokens': 9000,
    }
)

result = response.json()
print(result)
```

</CodeGroup>

## Response with Annotations

Web search responses include citation annotations:

```json
{
  "id": "resp_1234567890",
  "object": "response",
  "created_at": 1234567890,
  "model": "openai/o4-mini",
  "output": [
    {
      "type": "message",
      "id": "msg_abc123",
      "status": "completed",
      "role": "assistant",
      "content": [
        {
          "type": "output_text",
          "text": "OpenRouter is a unified API for accessing multiple Large Language Model providers through a single interface. It allows developers to access 100+ AI models from providers like OpenAI, Anthropic, Google, and others with intelligent routing and automatic failover.",
          "annotations": [
            {
              "type": "url_citation",
              "url": "https://openrouter.ai/docs",
              "start_index": 0,
              "end_index": 85
            },
            {
              "type": "url_citation",
              "url": "https://openrouter.ai/models",
              "start_index": 120,
              "end_index": 180
            }
          ]
        }
      ]
    }
  ],
  "usage": {
    "input_tokens": 15,
    "output_tokens": 95,
    "total_tokens": 110
  },
  "status": "completed"
}
```

## Annotation Types

Web search responses can include different annotation types:

### URL Citation
```json
{
  "type": "url_citation",
  "url": "https://example.com/article",
  "start_index": 0,
  "end_index": 50
}
```


## Complex Search Queries

Handle multi-part search queries:

<CodeGroup>

```typescript title="TypeScript"
const response = await fetch('https://openrouter.ai/api/v1/responses', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer YOUR_OPENROUTER_API_KEY',
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    model: 'openai/o4-mini',
    input: [
      {
        type: 'message',
        role: 'user',
        content: [
          {
            type: 'input_text',
            text: 'Compare OpenAI and Anthropic latest models',
          },
        ],
      },
    ],
    plugins: [{ id: 'web', max_results: 5 }],
    max_output_tokens: 9000,
  }),
});

const result = await response.json();
console.log(result);
```

```python title="Python"
import requests

response = requests.post(
    'https://openrouter.ai/api/v1/responses',
    headers={
        'Authorization': 'Bearer YOUR_OPENROUTER_API_KEY',
        'Content-Type': 'application/json',
    },
    json={
        'model': 'openai/o4-mini',
        'input': [
            {
                'type': 'message',
                'role': 'user',
                'content': [
                    {
                        'type': 'input_text',
                        'text': 'Compare OpenAI and Anthropic latest models',
                    },
                ],
            },
        ],
        'plugins': [{'id': 'web', 'max_results': 5}],
        'max_output_tokens': 9000,
    }
)

result = response.json()
print(result)
```

</CodeGroup>

## Web Search in Conversation

Include web search in multi-turn conversations:

<CodeGroup>

```typescript title="TypeScript"
const response = await fetch('https://openrouter.ai/api/v1/responses', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer YOUR_OPENROUTER_API_KEY',
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    model: 'openai/o4-mini',
    input: [
      {
        type: 'message',
        role: 'user',
        content: [
          {
            type: 'input_text',
            text: 'What is the latest version of React?',
          },
        ],
      },
      {
        type: 'message',
        id: 'msg_1',
        status: 'in_progress',
        role: 'assistant',
        content: [
          {
            type: 'output_text',
            text: 'Let me search for the latest React version.',
            annotations: [],
          },
        ],
      },
      {
        type: 'message',
        role: 'user',
        content: [
          {
            type: 'input_text',
            text: 'Yes, please find the most recent information',
          },
        ],
      },
    ],
    plugins: [{ id: 'web', max_results: 2 }],
    max_output_tokens: 9000,
  }),
});

const result = await response.json();
console.log(result);
```

```python title="Python"
import requests

response = requests.post(
    'https://openrouter.ai/api/v1/responses',
    headers={
        'Authorization': 'Bearer YOUR_OPENROUTER_API_KEY',
        'Content-Type': 'application/json',
    },
    json={
        'model': 'openai/o4-mini',
        'input': [
            {
                'type': 'message',
                'role': 'user',
                'content': [
                    {
                        'type': 'input_text',
                        'text': 'What is the latest version of React?',
                    },
                ],
            },
            {
                'type': 'message',
                'id': 'msg_1',
                'status': 'in_progress',
                'role': 'assistant',
                'content': [
                    {
                        'type': 'output_text',
                        'text': 'Let me search for the latest React version.',
                        'annotations': [],
                    },
                ],
            },
            {
                'type': 'message',
                'role': 'user',
                'content': [
                    {
                        'type': 'input_text',
                        'text': 'Yes, please find the most recent information',
                    },
                ],
            },
        ],
        'plugins': [{'id': 'web', 'max_results': 2}],
        'max_output_tokens': 9000,
    }
)

result = response.json()
print(result)
```

</CodeGroup>

## Streaming Web Search

Monitor web search progress with streaming:

<CodeGroup>

```typescript title="TypeScript"
const response = await fetch('https://openrouter.ai/api/v1/responses', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer YOUR_OPENROUTER_API_KEY',
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    model: 'openai/o4-mini',
    input: [
      {
        type: 'message',
        role: 'user',
        content: [
          {
            type: 'input_text',
            text: 'What is the latest news about AI?',
          },
        ],
      },
    ],
    plugins: [{ id: 'web', max_results: 2 }],
    stream: true,
    max_output_tokens: 9000,
  }),
});

const reader = response.body?.getReader();
const decoder = new TextDecoder();

while (true) {
  const { done, value } = await reader.read();
  if (done) break;

  const chunk = decoder.decode(value);
  const lines = chunk.split('\n');

  for (const line of lines) {
    if (line.startsWith('data: ')) {
      const data = line.slice(6);
      if (data === '[DONE]') return;

      try {
        const parsed = JSON.parse(data);
        if (parsed.type === 'response.output_item.added' &&
            parsed.item?.type === 'message') {
          console.log('Message added');
        }
        if (parsed.type === 'response.completed') {
          const annotations = parsed.response?.output
            ?.find(o => o.type === 'message')
            ?.content?.find(c => c.type === 'output_text')
            ?.annotations || [];
          console.log('Citations:', annotations.length);
        }
      } catch (e) {
        // Skip invalid JSON
      }
    }
  }
}
```

```python title="Python"
import requests
import json

response = requests.post(
    'https://openrouter.ai/api/v1/responses',
    headers={
        'Authorization': 'Bearer YOUR_OPENROUTER_API_KEY',
        'Content-Type': 'application/json',
    },
    json={
        'model': 'openai/o4-mini',
        'input': [
            {
                'type': 'message',
                'role': 'user',
                'content': [
                    {
                        'type': 'input_text',
                        'text': 'What is the latest news about AI?',
                    },
                ],
            },
        ],
        'plugins': [{'id': 'web', 'max_results': 2}],
        'stream': True,
        'max_output_tokens': 9000,
    },
    stream=True
)

for line in response.iter_lines():
    if line:
        line_str = line.decode('utf-8')
        if line_str.startswith('data: '):
            data = line_str[6:]
            if data == '[DONE]':
                break
            try:
                parsed = json.loads(data)
                if (parsed.get('type') == 'response.output_item.added' and
                    parsed.get('item', {}).get('type') == 'message'):
                    print('Message added')
                if parsed.get('type') == 'response.completed':
                    output = parsed.get('response', {}).get('output', [])
                    message = next((o for o in output if o.get('type') == 'message'), {})
                    content = message.get('content', [])
                    text_content = next((c for c in content if c.get('type') == 'output_text'), {})
                    annotations = text_content.get('annotations', [])
                    print(f'Citations: {len(annotations)}')
            except json.JSONDecodeError:
                continue
```

</CodeGroup>

## Annotation Processing

Extract and process citation information:

<CodeGroup>

```typescript title="TypeScript"
function extractCitations(response: any) {
  const messageOutput = response.output?.find((o: any) => o.type === 'message');
  const textContent = messageOutput?.content?.find((c: any) => c.type === 'output_text');
  const annotations = textContent?.annotations || [];

  return annotations
    .filter((annotation: any) => annotation.type === 'url_citation')
    .map((annotation: any) => ({
      url: annotation.url,
      text: textContent.text.slice(annotation.start_index, annotation.end_index),
      startIndex: annotation.start_index,
      endIndex: annotation.end_index,
    }));
}

const result = await response.json();
const citations = extractCitations(result);
console.log('Found citations:', citations);
```

```python title="Python"
def extract_citations(response_data):
    output = response_data.get('output', [])
    message_output = next((o for o in output if o.get('type') == 'message'), {})
    content = message_output.get('content', [])
    text_content = next((c for c in content if c.get('type') == 'output_text'), {})
    annotations = text_content.get('annotations', [])
    text = text_content.get('text', '')

    citations = []
    for annotation in annotations:
        if annotation.get('type') == 'url_citation':
            citations.append({
                'url': annotation.get('url'),
                'text': text[annotation.get('start_index', 0):annotation.get('end_index', 0)],
                'start_index': annotation.get('start_index'),
                'end_index': annotation.get('end_index'),
            })

    return citations

result = response.json()
citations = extract_citations(result)
print(f'Found citations: {citations}')
```

</CodeGroup>

## Best Practices

1. **Limit results**: Use appropriate `max_results` to balance quality and speed
2. **Handle annotations**: Process citation annotations for proper attribution
3. **Query specificity**: Make search queries specific for better results
4. **Error handling**: Handle cases where web search might fail
5. **Rate limits**: Be mindful of search rate limits


## Next Steps

- Learn about [Tool Calling](./tool-calling) integration
- Explore [Reasoning](./reasoning) capabilities
- Review [Basic Usage](./basic-usage) fundamentals

