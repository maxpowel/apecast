use meilisearch_sdk::{Client, search::SearchResult};
use serde::{Serialize, Deserialize};
use anyhow::Result;
use std::cmp::{max, min};

pub struct Search {
    client: Client,
    http: reqwest::Client
}

impl Search {
    pub fn new(uri: &str, token: &str) -> Search {
        let client = Client::new(uri, Some(token));
        Search {
            client,
            http: reqwest::ClientBuilder::new().build().unwrap()
        }
    }

    pub async fn search(&self, query: &str) -> Result<Vec<Article>>{
        let res = self.http.get(format!("https://forum.apecoin.com/search/query?term={}", query)).header("x-requested-with",  "XMLHttpRequest").send().await?;
        let results: ForumResponse = res.json().await?;
        let mut topics: Vec<Article> = results.topics.unwrap_or_default().into_iter().map(|res| Article {
            id: format!("{}", res.id),
            body: res.title.to_owned(),
            title: res.title,
            url: format!("https://forum.apecoin.com/t/{}/{}",res.slug, res.id),
            thumb: "https://cdn.stamp.fyi/space/apecoin.eth?s=160&cb=ec19915e02892e80".to_owned()

        }).collect();
        
        let mut search: Vec<Article> = self.client.index("articles").search().with_query(query).with_limit(5).execute::<Article>().await?.hits.into_iter().map(|a| a.result).collect();
        topics.append(&mut search);
        Ok(topics)
    }

    pub async fn insert(&self, articles: &Vec<Article>) -> Result<()>{
        self.client.index("articles").add_documents(articles, Some("id")).await?;
        Ok(())
    }
    
    pub async fn delete(&self, uids: &Vec<String>) -> Result<()>{
        self.client.index("articles").delete_documents(uids).await?;
        Ok(())
    }

    pub async fn delete_all(&self) -> Result<()>{
        self.client.index("articles").delete_all_documents().await?;
        Ok(())
    }


}

#[derive(Serialize, Deserialize, Debug)]
pub struct Article {
    pub id: String,
    pub title: String,
    pub body: String,
    pub url: String,
    pub thumb: String,
}

impl Article {
    pub fn hint(&self, query: &str) -> String {
        if query.is_empty() {
            if self.body.len() > 30 {
                self.body[0..30].to_owned()    
            } else {
                self.body.to_owned()
            }
        } else {
            if let Some(first_word) = query.split(' ').into_iter().collect::<Vec<&str>>().first() {
                if let Some(pos) = self.body.find(first_word) {
                    let start = if pos > 15 {
                        pos - 15
                    } else {
                        0
                    };

                    let hint = &self.body[max(0, start)..min(self.body.len(), pos + 15)];
                    format!("{}...", hint)
                } else {
                    if self.body.len() > 50 {
                        self.body[0..50].to_owned()    
                    } else {
                        self.body.to_owned()
                    }
                }
            } else {
                if self.body.len() > 50 {
                    self.body[0..50].to_owned()    
                } else {
                    self.body.to_owned()
                }
            }
        }

    }
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Topic {
    pub id: i32,
    pub title: String,
    pub slug: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ForumResponse {
    pub topics: Option<Vec<Topic>>,
}
