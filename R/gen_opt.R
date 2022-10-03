
#' gen_opt: estimate best treatment assignments via a genetic algorithm
#'
#' @param po_mat matrix of potential outcomes / conditional means. Must be nx4, where n is the number of units.
#' @param n_treat the number of units that should be treated
#' @param n_iter number of iterations for the genetic algorithm
#' @param temperature_decay the geometric rate at which the number of mutations per generation decreases. (must be less than one)
#'
#'
#' @export
gen_opt <- function(po_mat, n_treat, n_iter=500, temperature_decay=.97) {
      po_1_t <- po_mat[,1]
      po_1_c <- po_mat[,2]
      po_2_t <- po_mat[,3]
      po_2_c <- po_mat[,4]

     .Call(wrap__gen_opt, po_1_t, po_1_c, po_2_t, po_2_c, n_treat, n_iter, temperature_decay)
}
