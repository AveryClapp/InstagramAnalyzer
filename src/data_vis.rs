use rusqlite::{Connection, Result};
use plotters::prelude::*;
use chrono::NaiveDateTime;

fn main() -> Result<()> {
    // Connect to the SQLite database
    let conn = Connection::open("avery_clapp03_data.db")?;

    // Query follower and following counts
    let follower_data = get_data(&conn, "follower_data")?;
    let following_data = get_data(&conn, "following_data")?;

    // Create the line chart
    create_chart(&follower_data, &following_data)?;

    Ok(())
}

fn get_data(conn: &Connection, table: &str) -> Result<Vec<(NaiveDateTime, i32)>> {
    let mut stmt = conn.prepare(&format!("SELECT timestamp, COUNT(*) FROM {} GROUP BY timestamp ORDER BY timestamp", table))?;
    let data = stmt.query_map([], |row| {
        Ok((
            NaiveDateTime::from_timestamp(row.get(0)?, 0),
            row.get(1)?
        ))
    })?;

    data.collect()
}

fn create_chart(follower_data: &[(NaiveDateTime, i32)], following_data: &[(NaiveDateTime, i32)]) -> Result<()> {
    let root = BitMapBackend::new("instagram_data_chart.png", (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let min_date = follower_data.iter()
        .chain(following_data.iter())
        .map(|&(date, _)| date)
        .min()
        .unwrap();
    let max_date = follower_data.iter()
        .chain(following_data.iter())
        .map(|&(date, _)| date)
        .max()
        .unwrap();

    let max_count = follower_data.iter()
        .chain(following_data.iter())
        .map(|&(_, count)| count)
        .max()
        .unwrap();

    let mut chart = ChartBuilder::on(&root)
        .caption("Instagram Followers and Following Over Time", ("sans-serif", 30).into_font())
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(min_date..max_date, 0..max_count)?;

    chart.configure_mesh().draw()?;

    chart.draw_series(LineSeries::new(
        follower_data.iter().map(|&(date, count)| (date, count)),
        &RED,
    ))?
    .label("Followers")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

    chart.draw_series(LineSeries::new(
        following_data.iter().map(|&(date, count)| (date, count)),
        &BLUE,
    ))?
    .label("Following")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

    chart.configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    root.present()?;

    Ok(())