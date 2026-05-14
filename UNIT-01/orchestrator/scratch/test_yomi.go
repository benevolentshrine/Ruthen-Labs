package main

import (
	"fmt"
	"unit01/clients"
)

func main() {
	yomi := clients.NewYomiClient()
	m, err := yomi.GetProjectMap("/Users/lichi/SumiLabs/suji")
	if err != nil {
		fmt.Printf("Error: %v\n", err)
	} else {
		fmt.Printf("ProjectMap:\n%s\n", m)
	}
}
