library(tidyverse)
library(microbenchmark)

rextendr::document()
devtools::load_all()

n <- 5*10^5
df <- tibble(
  po_1_t = 1.5 * rexp(n), po_1_c = rexp(n),
  po_2_t = rexp(n), po_2_c = 1.2 * rexp(n),
)


test_1 <- gen_opt(df$po_1_t, df$po_1_c, df$po_2_c, df$po_2_t, 5000, 1) %>%
    as.data.frame()
test_1$gen <- "1"
colnames(test_1) <- c("x", "y", "rank", "gen")

test_2 <- gen_opt(df$po_1_t, df$po_1_c, df$po_2_c, df$po_2_t, 5000, 50) %>%
    as.data.frame()
test_2$gen <- "50"
colnames(test_2) <- c("x", "y", "rank", "gen")

bind_rows(test_1, test_2) %>%
    ggplot(aes(x=x,y=y,col=gen)) +
        geom_point()

ggplot(test_2, aes(x=x,y=y,col=rank)) +
    geom_point() +
    facet_wrap(~rank)



df %>%
  ggplot(aes(x = x, y = y, col = -rank)) +
  geom_point()
