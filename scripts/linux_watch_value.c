#include <errno.h>
#include <limits.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

static volatile int watch_value = 1337;
static volatile unsigned long write_count = 0;
static volatile sig_atomic_t should_exit = 0;

static void handle_exit_signal(int signal_number) {
    (void)signal_number;
    should_exit = 1;
}

static void print_current_state(void) {
    printf("pid: %ld\n", (long)getpid());
    printf("watch_value address: %p\n", (void *)&watch_value);
    printf("watch_value current: %d\n", watch_value);
    printf("write_count current: %lu\n", write_count);
    fflush(stdout);
}

int main(void) {
    char input_buffer[128];

    setvbuf(stdout, NULL, _IONBF, 0);
    signal(SIGINT, handle_exit_signal);
    signal(SIGTERM, handle_exit_signal);

    puts("Squalr Linux watch-value target.");
    puts("Scan this process for the current i32 value, then type a new integer here.");
    print_current_state();

    while (!should_exit) {
        char *parse_end_pointer = NULL;
        long parsed_value;

        printf("\nset watch_value> ");

        if (fgets(input_buffer, sizeof(input_buffer), stdin) == NULL) {
            break;
        }

        if (should_exit) {
            break;
        }

        errno = 0;
        parsed_value = strtol(input_buffer, &parse_end_pointer, 10);

        if (parse_end_pointer == input_buffer) {
            puts("No integer was found.");
            continue;
        }

        while (*parse_end_pointer == ' ' || *parse_end_pointer == '\t' || *parse_end_pointer == '\r' || *parse_end_pointer == '\n') {
            parse_end_pointer++;
        }

        if (*parse_end_pointer != '\0') {
            puts("Trailing characters were ignored; enter a plain base-10 integer.");
            continue;
        }

        if (errno == ERANGE || parsed_value < INT_MIN || parsed_value > INT_MAX) {
            puts("Value is outside the i32 range.");
            continue;
        }

        watch_value = (int)parsed_value;
        write_count++;

        printf("watch_value set to %d at %p, write_count=%lu\n", watch_value, (void *)&watch_value, write_count);
    }

    puts("\nExiting watch-value target.");
    print_current_state();

    return 0;
}
