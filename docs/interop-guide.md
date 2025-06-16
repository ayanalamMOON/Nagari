# Nagari ↔ JavaScript Interoperability

Nagari provides seamless interoperability with JavaScript, allowing you to use any JavaScript library, framework, or API from Nagari code, and vice versa.

## Core Concepts

### 1. Automatic Value Conversion

Nagari automatically converts between Nagari and JavaScript values:

```nagari
# Nagari values automatically converted to JS
numbers = [1, 2, 3, 4, 5]
result = Math.max(...numbers)  # JS Math.max receives JS array

# JS values automatically converted to Nagari
response = await fetch("https://api.example.com/data")
data = await response.json()  # JS Promise/object becomes Nagari dict
```

### 2. Function Wrapping

JavaScript functions are automatically wrapped for Nagari compatibility:

```nagari
# JS function automatically wrapped
from "lodash" import { map, filter }

numbers = [1, 2, 3, 4, 5]
doubled = map(numbers, (x) => x * 2)  # Lodash map works seamlessly
```

### 3. Module Integration

Built-in modules are automatically available through the interop system:

```nagari
# These work automatically through interop:
from "console" import { log, error, warn }
from "Math" import { sin, cos, PI, random }
from "JSON" import { parse, stringify }
```

## React Integration

### Basic Components

```nagari
from "react" import React, { useState, useEffect }

def MyComponent(props):
    count, setCount = useState(0)

    useEffect(() => {
        document.title = f"Count: {count}"
    }, [count])

    return <div className={props.className}>
        <h1>Count: {count}</h1>
        <button onClick={() => setCount(count + 1)}>
            Increment
        </button>
    </div>

export default MyComponent
```

### Hooks

All React hooks work seamlessly:

```nagari
from "react" import { useState, useEffect, useCallback, useMemo, useRef }

def AdvancedComponent():
    # State management
    data, setData = useState([])
    loading, setLoading = useState(true)

    # Refs
    inputRef = useRef(None)

    # Memoized calculations
    processedData = useMemo(() => {
        return data.filter(item => item.active)
    }, [data])

    # Callbacks
    handleSubmit = useCallback((event) => {
        event.preventDefault()
        # Handle form submission
    }, [])

    # Effects
    useEffect(() => {
        async def loadData():
            setLoading(true)
            try:
                response = await fetch("/api/data")
                result = await response.json()
                setData(result)
            finally:
                setLoading(false)

        loadData()
    }, [])

    return <form onSubmit={handleSubmit}>
        {loading ? <div>Loading...</div> : processedData.map(item =>
            <div key={item.id}>{item.name}</div>
        )}
    </form>
```

## Node.js Integration

### Express Servers

```nagari
from "express" import express
from "cors" import cors

def createAPI():
    app = express()

    # Middleware
    app.use(cors())
    app.use(express.json())

    # Routes
    def getUsers(req, res):
        users = [
            {"id": 1, "name": "Alice"},
            {"id": 2, "name": "Bob"}
        ]
        res.json(users)

    def createUser(req, res):
        user = req.body
        # Save user logic here
        res.status(201).json(user)

    app.get("/users", getUsers)
    app.post("/users", createUser)

    return app

# Start server
app = createAPI()
app.listen(3000, () => {
    console.log("Server running on port 3000")
})
```

### File System Operations

```nagari
from "fs/promises" import { readFile, writeFile, mkdir }
from "path" import { join, dirname }

async def processFiles(inputDir: str, outputDir: str):
    try:
        # Read all files
        files = await fs.readdir(inputDir)

        for filename in files:
            if filename.endswith(".txt"):
                # Read file
                content = await readFile(join(inputDir, filename), "utf8")

                # Process content
                processed = content.upper()

                # Write to output
                await mkdir(outputDir, {"recursive": true})
                await writeFile(join(outputDir, filename), processed)

        console.log(f"Processed {len(files)} files")

    except Exception as error:
        console.error(f"Error processing files: {error}")
```

## Web APIs

### Fetch and HTTP

```nagari
# Fetch API with automatic async/await
async def apiRequest(endpoint: str, options: dict = {}):
    try:
        response = await fetch(endpoint, options)

        if not response.ok:
            raise Exception(f"HTTP {response.status}: {response.statusText}")

        return await response.json()

    except Exception as error:
        console.error(f"API request failed: {error}")
        return None

# Usage
data = await apiRequest("/api/users", {
    "method": "POST",
    "headers": {"Content-Type": "application/json"},
    "body": JSON.stringify({"name": "Alice"})
})
```

### DOM Manipulation

```nagari
# DOM queries and manipulation
def setupPage():
    # Query elements
    header = document.querySelector("h1")
    buttons = document.querySelectorAll(".btn")

    # Create elements
    newDiv = document.createElement("div")
    newDiv.className = "container"
    newDiv.innerHTML = "<p>Created with Nagari!</p>"

    # Event listeners
    for button in buttons:
        button.addEventListener("click", (event) => {
            console.log(f"Button {button.textContent} clicked!")
            event.target.classList.toggle("active")
        })

    # Append to DOM
    document.body.appendChild(newDiv)

# Setup when DOM is ready
if document.readyState == "loading":
    document.addEventListener("DOMContentLoaded", setupPage)
else:
    setupPage()
```

## Custom JavaScript Integration

### Using Any npm Package

```nagari
# Install any npm package and use it directly
from "moment" import moment
from "axios" import axios
from "lodash" import { map, filter, groupBy }

# Date manipulation
now = moment()
formatted = now.format("YYYY-MM-DD HH:mm:ss")

# HTTP client
async def getData():
    response = await axios.get("https://api.example.com/data")
    return response.data

# Functional utilities
data = [1, 2, 3, 4, 5, 6]
evens = filter(data, x => x % 2 == 0)
doubled = map(evens, x => x * 2)
```

### Direct JavaScript Code

For complex interop scenarios, you can embed JavaScript directly:

```nagari
# Direct JavaScript integration
jsHelper = js"""
    function complexCalculation(data) {
        // Complex JS logic here
        return result;
    }
"""

result = jsHelper.complexCalculation(myData)
```

## Type Safety

Nagari provides type hints for better JavaScript interop:

```nagari
from "react" import { FC, ReactNode } from "@types/react"

def TypedComponent(props: {
    title: str,
    children: ReactNode,
    onClick: (event: Event) => None
}): ReactNode:
    return <div onClick={props.onClick}>
        <h1>{props.title}</h1>
        {props.children}
    </div>
```

## Best Practices

### 1. Prefer Nagari Syntax

```nagari
# Good: Use Nagari syntax when possible
data = [1, 2, 3, 4, 5]
result = max(data)

# Avoid: Direct JS syntax unless necessary
result = js("Math.max(...[1, 2, 3, 4, 5])")
```

### 2. Handle Async Properly

```nagari
# Good: Use async/await consistently
async def fetchAndProcess():
    data = await fetch("/api/data")
    json_data = await data.json()
    return process(json_data)

# Avoid: Mixing Promise chains
def fetchAndProcess():
    return fetch("/api/data").then(r => r.json()).then(process)
```

### 3. Use Type Hints

```nagari
# Good: Specify types for interop functions
def processUser(user: dict[str, any]) -> dict[str, str]:
    return {"name": user["name"], "email": user["email"]}

# Better: Use specific interfaces when available
from "@types/user" import User
def processUser(user: User) -> UserSummary:
    return {"name": user.name, "email": user.email}
```

## Error Handling

JavaScript errors are automatically converted to Nagari exceptions:

```nagari
try:
    result = await fetch("/api/data")
    data = await result.json()
except FetchError as e:
    console.error(f"Network error: {e}")
except JSONError as e:
    console.error(f"JSON parsing error: {e}")
except Exception as e:
    console.error(f"Unexpected error: {e}")
```

## Performance Considerations

1. **Value Conversion**: Automatic conversion has minimal overhead
2. **Function Wrapping**: Wrapped functions are cached for reuse
3. **Memory Management**: Nagari runtime handles garbage collection coordination
4. **Bundle Size**: Only used interop modules are included in the final bundle

The interop system makes JavaScript ecosystem integration seamless while maintaining Nagari's clean, Pythonic syntax!
