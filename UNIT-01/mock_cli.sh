#!/usr/bin/env bash
# ==============================================================================
# RUTHEN LABS - UNIT-01 SOVEREIGN CLI (v2.7 INDUSTRIAL)
# ==============================================================================
set -euo pipefail

# Theme: Sovereign Industrial
AMBER='\033[38;5;214m'
GREY='\033[38;5;244m'
DIM='\033[38;5;240m'
WHITE='\033[38;5;255m'
GREEN_BG='\033[48;5;22m\033[38;5;255m'
RED_BG='\033[48;5;52m\033[38;5;255m'
BG_AMBER='\033[48;5;214m\033[38;5;232m'
NC='\033[0m' # No Color

# Dynamically detect terminal width
WIDTH=$(tput cols || echo 80)
HR=$(printf '%*s' "$WIDTH" '' | tr ' ' '─')

# Cleanup on exit
cleanup() {
    tput cnorm # Show cursor
}
trap cleanup EXIT

clear

# 1. THE HEADER SHIELD
echo -e "${BG_AMBER} UNIT-01 v1.5.0 ${NC}${AMBER} █ ${NC}${GREY} ruthen-labs/engine ${NC}"
echo -e "${DIM}${HR}${NC}"
echo -e " ${AMBER}●${NC} ARCHITECT MODE  ${DIM}│${NC} 32GB RAM  ${DIM}│${NC} INDEXER: ${AMBER}ONLINE${NC}  ${DIM}│${NC} SANDBOX: ${AMBER}ONLINE${NC}"
echo -e "${DIM}${HR}${NC}\n"

# 2. SOVEREIGN INPUT
echo -e "${GREY}waiting for directive...${NC}"
echo -ne " ${AMBER}»${NC} "
read -r INPUT
echo -e "${DIM}${HR}${NC}"

# 3. OPERATION LOG
echo -e "\n${WHITE}Refactoring main.rs${NC}"
echo -e "${DIM}│${NC} ${GREY}Objective: Modularize greeting logic for zero-trust compliance.${NC}"

# 4. INDUSTRIAL DIRECTIVE CARD
echo -e "\n${AMBER}█${NC}${BG_AMBER} WRITE FILE ${NC} ${WHITE}src/main.rs${NC}"
echo -e "${AMBER}┃${NC}"
echo -e "${AMBER}┃${NC} ${DIM}<sandbox_write path=\"src/main.rs\">${NC}"

# Diff Section
echo -e "${AMBER}┃${NC} ${DIM}1${NC}  fn main() {"
echo -e "${AMBER}┃${NC} ${RED_BG} 2 -    println!(\"Hello World\");                                           ${NC}"
echo -e "${AMBER}┃${NC} ${GREEN_BG} 3 +    let greeting = get_greeting();                                     ${NC}"
echo -e "${AMBER}┃${NC} ${GREEN_BG} 4 +    println!(\"{}\", greeting);                                         ${NC}"
echo -e "${AMBER}┃${NC} ${DIM}5${NC}  }"
echo -e "${AMBER}┃${NC} ${DIM}</sandbox_write>${NC}"
echo -e "${AMBER}┃${NC}"

# 5. INTERACTIVE REVIEW GATE
echo -e "${WHITE}Review required for disk operation:${NC}"
options=("Authorize Write" "Session Trust" "External Edit" "Reject & Refactor")
current=0

tput sc # Save cursor
tput civis # Hide cursor

draw_menu() {
    tput rc # Restore cursor
    for i in "${!options[@]}"; do
        tput el # Clear line
        if [ "$i" -eq "$current" ]; then
            echo -e " ${AMBER}█${NC} ${WHITE}${options[$i]}${NC}"
        else
            echo -e "   ${DIM}${options[$i]}${NC}"
        fi
    done
}

draw_menu

while true; do
    read -rsn1 key
    if [[ "$key" == $'\x1b' ]]; then
        read -rsn2 key
        if [[ "$key" == "[A" ]]; then # Up
            if [ "$current" -gt 0 ]; then
                current=$((current-1))
                draw_menu
            fi
        elif [[ "$key" == "[B" ]]; then # Down
            if [ "$current" -lt 3 ]; then
                current=$((current+1))
                draw_menu
            fi
        fi
    elif [[ "$key" == "" ]]; then # Enter
        break
    fi
done

tput cnorm # Show cursor

# 6. STATUS STAMPS
STAMP_POS=$((WIDTH - 20))
echo -e "\n${AMBER}█${NC} OPERATION SUCCESSFUL"
echo -e "  ${GREY}File written: ${NC}${WHITE}src/main.rs${NC}"
echo -e "  ${GREY}Checksum:     ${NC}${DIM}ae32..f41b${NC}"

echo -e "\n${AMBER}✦${NC} ${WHITE}System state synchronized. Next operation ready.${NC}\n"
echo -ne " ${AMBER}unit01${NC} ${DIM}»${NC} "
sleep 1
echo ""
