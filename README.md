# spending-tracker-api
If I were to balance a checkbook, I'd do it using this.

This application runs on port 8001.

Routes:
* GET/POST `/spent` to view total or add to it with a request like:
```json
{
  "amount": 39.95,
  "category": "Dining"
}
```
* GET `/reset` to reset the running total to zero.
* GET `/` to view the pie chart which is generated with [Chart.js](https://www.chartjs.org/)

