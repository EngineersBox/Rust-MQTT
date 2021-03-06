BLACK:=$(shell tput setaf 0)
RED:=$(shell tput setaf 1)
GREEN:=$(shell tput setaf 2)
YELLOW:=$(shell tput setaf 3)
BLUE:=$(shell tput setaf 4)
MAGENTA:=$(shell tput setaf 5)
CYAN:=$(shell tput setaf 6)
WHITE:=$(shell tput setaf 7)

BOLD:=$(shell tput bold)
ULINE:=$(call tput smul)

RESET := $(shell tput sgr0)

SRC:=src/main.rs
TARGET_DIR:=target
OUT:=$(TARGET_DIR)/debug/rust_mqtt
LOGS_DIR=logs
DATE:=$(shell date +"%Y-%m-%d_%H-%M-%S")

.PHONY: clean_logs clean_log_archives archive_logs clean_target build check run_pubcontroller run_analyser

clean_logs:
	@echo '$(CYAN)>> $(GREEN) Removing logs in $(WHITE)[$(RED)$(BOLD)$(LOGS_DIR)/*$(RESET)$(WHITE)]'
	@echo '--------------------------------'
ifeq (,$(wildcard $(LOGS_DIR)/*.log))
	@echo "$(RED)$(BOLD)No files to remove$(RESET)"
else
	-@rm -rvf $(LOGS_DIR)/*.log | xargs -L 1 echo " [$(RED)-$(WHITE)]"
endif
	@echo '--------------------------------'

clean_log_archives:
	@echo '$(CYAN)>> $(GREEN) Removing archives in $(WHITE)[$(RED)$(BOLD)$(LOGS_DIR)/*$(RESET)$(WHITE)]'
	@echo '--------------------------------'
ifeq (,$(wildcard $(LOGS_DIR)/*.zip))
	@echo "$(RED)$(BOLD)No files to remove$(RESET)"
else
	-@rm -rvf $(LOGS_DIR)/*.zip | xargs -L 1 echo " [$(RED)-$(WHITE)]"
endif
	@echo '--------------------------------'

archive_logs:
	@echo '$(CYAN)>> $(GREEN) Archiving logs in $(WHITE)[$(RED)$(BOLD)$(LOGS_DIR)$(RESET)$(WHITE)]'
	@echo '--------------------------------'
ifeq (,$(wildcard $(LOGS_DIR)/*.log))
	@echo "$(RED)$(BOLD)No logs to archive$(RESET)"
else
	-@zip -pr logs/archive-$(DATE).zip $(LOGS_DIR)/*.log | xargs -L 1 echo " [$(GREEN)+$(WHITE)] $0 to archive"
endif
	@echo '--------------------------------'
	@make -f Makefile clean_logs

clean_target:
	@echo '$(CYAN)>> $(GREEN) Removing previous builds $(WHITE)[$(RED)$(BOLD)$(OUT)$(RESET)$(WHITE)]'
	@echo '--------------------------------'
	@cargo clean
	@echo '--------------------------------'

build:
	@echo '$(CYAN)>> $(GREEN) Compiling build $(WHITE)[$(RED)$(BOLD)$(OUT)$(RESET)$(WHITE)]'
	@echo '--------------------------------'
	@cargo build -v --color always
	@echo '--------------------------------'
	@echo '$(CYAN)>> $(GREEN) Finished compiling to $(TARGET_DIR)$(RESET)'

check:
	@echo '$(CYAN)>> $(GREEN) Checking build $(WHITE)[$(RED)$(BOLD)$(OUT)$(RESET)$(WHITE)]'
	@echo '--------------------------------'
	@cargo check -v --color always
	@echo '--------------------------------'
	@echo '$(CYAN)>> $(GREEN) Finished checking build $(RESET)'

run_pubcontroller:
	@echo '$(CYAN)>> $(GREEN) Running new build $(WHITE)[$(RED)$(BOLD)$(OUT)$(RESET)$(WHITE)]'
	@echo '--------------------------------'
	@cargo run --color always --bin pubcontroller
	@echo '--------------------------------'
	@echo '$(CYAN)>> $(GREEN) Stopped running$(RESET)'

run_analyser:
	@echo '$(CYAN)>> $(GREEN) Running new build $(WHITE)[$(RED)$(BOLD)$(OUT)$(RESET)$(WHITE)]'
	@echo '--------------------------------'
	@cargo run --color always --bin analyser
	@echo '--------------------------------'
	@echo '$(CYAN)>> $(GREEN) Stopped running$(RESET)'