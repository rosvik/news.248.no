use crate::{db, nrk, rss};
use tokio_cron_scheduler::{Job, JobScheduler};

// Every 15 minutes
const CRON_STRING: &str = "0 1/15 * * * *";

pub async fn start_scheduler() -> Result<JobScheduler, Box<dyn std::error::Error>> {
    let mut scheduler = JobScheduler::new().await?;

    let nrk_job = Job::new_async(CRON_STRING, |_, _| {
        Box::pin(async move {
            let articles = nrk::nrk("https://www.nrk.no/nyheter").await;
            println!("Articles: {:?}", articles);
            for article in articles {
                db::add_article(article, db::NRK_ID);
            }
        })
    })?;
    let bbc_job = Job::new_async(CRON_STRING, |_, _| {
        Box::pin(async move {
            let articles = rss::rss("https://feeds.bbci.co.uk/news/world/rss.xml").await;
            println!("Articles: {:?}", articles);
            for article in articles {
                db::add_article(article, db::BBC_ID);
            }
        })
    })?;
    scheduler.add(nrk_job).await?;
    scheduler.add(bbc_job).await?;

    scheduler.start().await?;
    if let Ok(Some(next_job_in)) = scheduler.time_till_next_job().await {
        println!("Scheduler started, next job in {next_job_in:?}");
    }
    Ok(scheduler)
}
