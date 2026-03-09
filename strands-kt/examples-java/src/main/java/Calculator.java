import com.strands.Agent;
import com.strands.ModelConfig;
import com.strands.StreamEvent;
import com.strands.Tool;
import java.util.List;

public class Calculator {
    public static void main(String[] args) {
        var calculator = new Tool(
            "calculator",
            "Evaluate a math expression and return the numeric result.",
            "{\"type\":\"object\",\"properties\":{\"expression\":{\"type\":\"string\",\"description\":\"A math expression to evaluate\"}},\"required\":[\"expression\"]}",
            (input, toolUseId) -> "{\"status\":\"success\",\"content\":[{\"text\":\"714\"}]}"
        );

        try (var agent = new Agent(
            ModelConfig.bedrock(),
            "You are a helpful assistant with a calculator tool.",
            List.of(calculator)
        )) {
            agent.forEachEvent("What is 42 * 17?", event -> {
                if (event instanceof StreamEvent.TextDelta td) {
                    System.out.print(td.getText());
                } else if (event instanceof StreamEvent.Stop stop) {
                    System.out.println("\n[" + stop.getReason() + "]");
                } else if (event instanceof StreamEvent.ToolUse tu) {
                    System.out.println("[tool-use: " + tu.getName() + "]");
                } else if (event instanceof StreamEvent.ToolResult tr) {
                    System.out.println("[tool-result: " + tr.getStatus() + " => " + tr.getContent() + "]");
                }
            }).join();
        }
    }
}
