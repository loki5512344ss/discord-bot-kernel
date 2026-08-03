package kernel.loki.embed;

import static org.junit.jupiter.api.Assertions.*;
import org.junit.jupiter.api.Test;
import net.dv8tion.jda.api.entities.MessageEmbed;

class EmbedFactoryTest {
    @Test
    void shouldCreateErrorEmbed() {
        MessageEmbed embed = EmbedTemplates.error("Test Error", "This is a test error");
        assertEquals("Test Error", embed.getTitle());
        assertEquals("This is a test error", embed.getDescription());
    }

    @Test
    void shouldCreateSuccessEmbed() {
        MessageEmbed embed = EmbedTemplates.success("Success", "Operation completed");
        assertEquals("Success", embed.getTitle());
        assertEquals("Operation completed", embed.getDescription());
    }

    @Test
    void shouldCreateInfoEmbed() {
        MessageEmbed embed = EmbedTemplates.info("Info", "Some info");
        assertNotNull(embed);
    }

    @Test
    void shouldCreateWarningEmbed() {
        MessageEmbed embed = EmbedTemplates.warning("Warning", "Be careful");
        assertNotNull(embed);
    }
}
