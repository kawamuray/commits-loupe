use super::CommitViewData;
use crate::commit::CommitInfo;
use std::rc::Rc;
use yew::prelude::*;

/// A table component which shows each commit's summary
pub struct TableComponent {
    link: ComponentLink<Self>,
    props: Properties,
}

#[derive(Debug, Clone, Properties)]
pub struct Properties {
    pub data: Option<Rc<CommitViewData>>,
    pub query: String,
}

impl TableComponent {
    fn view_commit_table_entry(data: &CommitViewData, commit: &CommitInfo) -> Html {
        let throughput = data.metadata.get(&commit.sha).expect("missing meta value");
        html! {
          <tr>
            <th><a href=commit.view_url.clone()>{ &commit.sha }</a></th>
            <th>{ &commit.author.name }</th>
            <th>{ "&commit.author_date" }</th>
            <th>{ &commit.message }</th>
            <th>{ throughput }</th>
          </tr>
        }
    }

    fn view_commit_table_entries(&self) -> Vec<Html> {
        if let Some(data) = self.props.data.as_ref() {
            let mut htmls = Vec::with_capacity(data.commits.len());
            for commit in &data.commits {
                htmls.push(Self::view_commit_table_entry(&data, commit));
            }
            htmls
        } else {
            vec![html! {}]
        }
    }
}

impl Component for TableComponent {
    type Message = ();
    type Properties = Properties;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { link, props }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        true
    }

    fn view(&self) -> Html {
        html! {
          <table>
            <tr>
              <th>{ "Commit" }</th>
              <th>{ "Author" }</th>
              <th>{ "Timestamp" }</th>
              <th>{ "Subject" }</th>
              <th>{ "Throughput" }</th>
            </tr>
            { for self.view_commit_table_entries() }
          </table>
        }
    }
}
