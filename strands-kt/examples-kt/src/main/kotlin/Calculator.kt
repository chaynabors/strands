import com.strands.Agent
import com.strands.ModelConfig
import com.strands.StreamEvent
import com.strands.Tool
import kotlinx.coroutines.runBlocking

fun main() = runBlocking {
    val calculator = Tool(
        name = "calculator",
        description = "Evaluate a math expression and return the numeric result.",
        inputSchema = """{"type":"object","properties":{"expression":{"type":"string","description":"A math expression to evaluate"}},"required":["expression"]}""",
    ) { _, _ ->
        """{"status":"success","content":[{"text":"714"}]}"""
    }

    Agent(
        model = ModelConfig.Bedrock(),
        systemPrompt = "You are a helpful assistant with a calculator tool.",
        tools = listOf(calculator),
    ).use { agent ->
        agent.stream("What is 42 * 17?").collect { event ->
            when (event) {
                is StreamEvent.TextDelta -> print(event.text)
                is StreamEvent.Stop -> println("\n[${event.reason}]")
                is StreamEvent.ToolUse -> println("[tool-use: ${event.name}]")
                is StreamEvent.ToolResult -> println("[tool-result: ${event.status} => ${event.content}]")
                else -> {}
            }
        }
    }
}
