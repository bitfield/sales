package main

import (
	"encoding/csv"
	"fmt"
	"io"
	"log"
	"os"
	"strconv"
)

type USD int

func NewUSD(dollars float64) USD {
	return USD(dollars * 100)
}

func (u USD) Dollars() float64 {
	return float64(u) / 100
}

func main() {
	units := map[string]int{}
	revenue := map[string]USD{}
	productWidth := 0
	f, err := os.Open(os.Args[1])
	if err != nil {
		log.Fatal(err)
	}
	defer f.Close()
	r := csv.NewReader(f)
	for {
		record, err := r.Read()
		if err == io.EOF {
			break
		}
		if err != nil {
			log.Fatal(err)
		}
		if record[0] == "Order ID" {
			continue
		}
		// fmt.Printf("%#v\n", record)
		product := record[17]
		if len(product) > productWidth {
			productWidth = len(product)
		}
		units[product]++
		price, err := strconv.ParseFloat(record[18], 64)
		if err != nil {
			line, col := r.FieldPos(18)
			log.Fatal("line", line, "col", col, err)
		}
		revenue[product] += NewUSD(price)
	}
	var totalRevenue USD
	var totalUnits int
	for product, u := range units {
		fmt.Printf("%-*s %d %.2f\n", productWidth, product, u, revenue[product].Dollars())
		totalRevenue += revenue[product]
		totalUnits += u
	}
	fmt.Println("Total revenue", totalRevenue.Dollars())
	fmt.Println("Total units", totalUnits)
}
